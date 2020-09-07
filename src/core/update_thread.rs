use std::convert::TryFrom;
use std::mem::{MaybeUninit, replace};
use std::panic::AssertUnwindSafe;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use ctclient::{CTClient, SignedTreeHead, SthResult};
use ctclient::internal::Leaf;
use diesel::prelude::*;
use thiserror::Error;

use crate::core::db::{DBPool, DBPooledConn, PgConnectionHelper};
use crate::models::{CtLog, Hash, Sth};

enum ChannelMessage {
  Stop
}

pub struct Handle {
  jh: MaybeUninit<JoinHandle<()>>,
  sender: mpsc::Sender<ChannelMessage>,
}

impl Drop for Handle {
  fn drop(&mut self) {
    self.sender.send(ChannelMessage::Stop).unwrap();
    unsafe { replace(&mut self.jh, MaybeUninit::uninit()).assume_init() }.join().unwrap();
  }
}

trait _DbErrorUnwrapHelper {
  type Inner;
  fn unwrap_or_display_err(self) -> Self::Inner;
}
impl<T> _DbErrorUnwrapHelper for Result<T, diesel::result::Error> {
  type Inner = T;

  fn unwrap_or_display_err(self) -> T {
    match self {
      Ok(t) => t,
      Err(e) => panic!("Database error: {}", e)
    }
  }
}

pub fn init_thread(db_pool: &'static DBPool, log: CtLog) -> Handle {
  let (sender, recv) = mpsc::channel::<ChannelMessage>();
  let jh = thread::Builder::new().name(format!("update-{}", &log.log_id)).spawn(move || {
    match std::panic::catch_unwind(AssertUnwindSafe(move || {
      macro_rules! get_db {
        () => {
          match db_pool.get() {
            Ok(k) => k,
            Err(e) => panic!("Error opening db: {}", e)
          }
        };
      }

      use diesel::prelude::*;
      use crate::schema::ctlogs::dsl::*;
      use crate::schema::sth::dsl::*;
      use crate::schema::sth::dsl::id as sth_id;
      use crate::schema::ctlogs::dsl::log_id as ctlogs_log_id;
      use crate::schema::sth::dsl::log_id as sth_log_id;

      let http_client = ctclient::internal::new_http_client().unwrap();
      let parsed_url = ctclient::internal::re_exports::reqwest::Url::parse(&log.endpoint_url).unwrap();
      let parsed_pub_key = ctclient::internal::re_exports::openssl::pkey::PKey::public_key_from_der(&log.public_key.0).unwrap();
      struct FetchedSth {
        stored_as_id: i64,
        sth: SignedTreeHead,
      }
      #[derive(Debug, Error)]
      enum FetchSthError {
        #[error("{0}")]
        CtClient(#[from] ctclient::Error),
        #[error("Tree size larger than i64::MAX are not supported.")]
        TreeSizeTooLarge
      }
      let fetch_sth = |db: &DBPooledConn| -> Result<FetchedSth, FetchSthError> {
        let th = ctclient::internal::check_tree_head(
          &http_client,
          &parsed_url,
          &parsed_pub_key,
        )?;
        if th.tree_size > i64::MAX as u64 {
          return Err(FetchSthError::TreeSizeTooLarge);
        }
        let ins = crate::models::inserts::Sth {
          log_id: log.log_id,
          tree_hash: Hash(th.root_hash),
          tree_size: th.tree_size as i64,
          sth_timestamp: th.timestamp as i64,
          signature: &th.signature[..],
          checked_consistent_with_latest: false,
        };
        let stored_as_id = db.transaction_rw_serializable::<i64, diesel::result::Error, _>(|| {
          let res: Vec<i64> = diesel::insert_into(sth)
              .values(&ins)
              .on_conflict_do_nothing()
              .returning(sth_id)
              .get_results(db)?;
          if res.is_empty() {
            // already exists
            let existing_id: Vec<i64> = sth.select(sth_id)
                .filter(
                  sth_log_id.eq(&log.log_id)
                      .and(tree_size.eq(ins.tree_size))
                      .and(tree_hash.eq(ins.tree_hash))
                      .and(sth_timestamp.eq(ins.sth_timestamp)))
                .load(db)?;
            Ok(existing_id[0])
          } else {
            Ok(res[0])
          }
        }).unwrap_or_display_err();
        Ok(FetchedSth {
          stored_as_id,
          sth: th
        })
      };

      let check_unchecked_consistency = |db: &DBPooledConn, latest: &FetchedSth| {
        let sth_to_check: Vec<Sth> =
            sth.filter(
              checked_consistent_with_latest.eq(false)
                  .and(sth_log_id.eq(&log.log_id))
                  .and(tree_size.le(latest.sth.tree_size as i64))
            ).load(db).unwrap_or_display_err();
        for s in sth_to_check {
          let s_id = s.id;
          let s = SignedTreeHead {
            tree_size: s.tree_size as u64,
            timestamp: s.sth_timestamp as u64,
            root_hash: s.tree_hash.0,
            signature: s.signature.0.clone()
          };
          use std::cmp::Ordering::*;
          let mut pass = false;
          match s.tree_size.cmp(&latest.sth.tree_size) {
            Greater => unreachable!(),
            Equal => {
              if s.root_hash == latest.sth.root_hash {
                pass = true;
              } else {
                crate::models::inserts::ConsistencyCheckError::upsert(
                  db,
                  log.log_id,
                  s_id,
                  latest.stored_as_id,
                  "Different hash but same tree size."
                ).unwrap_or_display_err();
              }
            },
            Less => {
              match ctclient::internal::check_consistency_proof(
                &http_client,
                &parsed_url,
                s.tree_size,
                latest.sth.tree_size,
                &s.root_hash,
                &latest.sth.root_hash
              ) {
                Ok(_) => {
                  pass = true;
                },
                Err(e) => {
                  crate::models::inserts::ConsistencyCheckError::upsert(
                    db,
                    log.log_id,
                    s_id,
                    latest.stored_as_id,
                    &format!("{}", e)
                  ).unwrap_or_display_err();
                }
              }
            }
          }
          if pass {
            diesel::update(sth)
                .filter(sth_id.eq(s_id))
                .set(checked_consistent_with_latest.eq(true))
                .execute(db).unwrap_or_display_err();
            {
              use crate::schema::consistency_check_errors::dsl;
              diesel::delete(dsl::consistency_check_errors)
                  .filter(
                    dsl::log_id.eq(&log.log_id)
                        .and(dsl::from_sth_id.eq(s_id))
                        .and(dsl::to_sth_id.eq(latest.stored_as_id))
                  )
                  .execute(db).unwrap_or_display_err();
            }
          }
        }
      };

      let advance_latest_sth = |db: &DBPooledConn, new_latest: &FetchedSth| {
        db.transaction_rw_serializable::<(), diesel::result::Error, _>(|| {
          diesel::update(sth)
              .filter(sth_id.eq(new_latest.stored_as_id))
              .set(checked_consistent_with_latest.eq(true))
              .execute(db)?;
          diesel::update(ctlogs)
              .filter(ctlogs_log_id.eq(&log.log_id))
              .set(latest_sth.eq(new_latest.stored_as_id))
              .execute(db)?;
          Ok(())
        }).unwrap_or_display_err();
        check_unchecked_consistency(db, new_latest);
      };

      let mut last_fetched_sth: Option<FetchedSth> = None;
      let mut current_db_hdl: Option<DBPooledConn> = Some(get_db!());
      if let Some(latest_sth_id) = log.latest_sth {
        let db = current_db_hdl.as_ref().unwrap();
        let mut stored_sth: Vec<Sth> = sth
            .filter(sth_id.eq(latest_sth_id))
            .load(db).unwrap_or_display_err();
        assert_eq!(stored_sth.len(), 1);
        assert!(stored_sth[0].checked_consistent_with_latest);
        let stored_sth = stored_sth.swap_remove(0);
        last_fetched_sth = Some(FetchedSth {
          stored_as_id: stored_sth.id,
          sth: SignedTreeHead {
            tree_size: stored_sth.tree_size as u64,
            timestamp: stored_sth.sth_timestamp as u64,
            root_hash: stored_sth.tree_hash.0,
            signature: stored_sth.signature.0
          }
        });
      }

      loop {
        if current_db_hdl.is_none() {
          current_db_hdl = Some(get_db!());
        }
        let db = current_db_hdl.as_ref().unwrap();
        let new_sth = match fetch_sth(db) {
          Ok(s) => {
            diesel::update(ctlogs)
                .filter(ctlogs_log_id.eq(&log.log_id))
                .set(last_sth_error.eq(None::<String>))
                .execute(db).unwrap_or_display_err();
            s
          },
          Err(e) => {
            diesel::update(ctlogs)
                .filter(ctlogs_log_id.eq(&log.log_id))
                .set(last_sth_error.eq(format!("{}", e)))
                .execute(db).unwrap_or_display_err();
            continue;
          }
        };
        match last_fetched_sth {
          None => {
            advance_latest_sth(db, &new_sth);
            last_fetched_sth = Some(new_sth);
          },
          Some(ref old_sth) => {
            use std::cmp::Ordering::*;
            match new_sth.sth.tree_size.cmp(&old_sth.sth.tree_size) {
              Less | Equal => {
                check_unchecked_consistency(db, &old_sth);
              },
              Greater => 'o : {
                let consistency_proof_parts_res = ctclient::internal::check_consistency_proof(
                  &http_client,
                  &parsed_url,
                  old_sth.sth.tree_size,
                  new_sth.sth.tree_size,
                  &old_sth.sth.root_hash,
                  &new_sth.sth.root_hash
                );
                if let Err(e) = consistency_proof_parts_res {
                  crate::models::inserts::ConsistencyCheckError::upsert(
                    db,
                    log.log_id,
                    old_sth.stored_as_id,
                    new_sth.stored_as_id,
                    &format!("{}", e),
                  ).unwrap_or_display_err();
                  break 'o;
                }
                use crate::schema::cert_fetch_errors::dsl as cfe;
                let consistency_proof_parts = consistency_proof_parts_res.unwrap();
                let mut leaf_hashs = Vec::with_capacity(usize::try_from(new_sth.sth.tree_size - old_sth.sth.tree_size).unwrap());
                let mut has_error = false;
                macro_rules! cfe_insert {
                    ($e:expr) => {
                      let ins = crate::models::inserts::CertFetchError {
                        log_id: log.log_id,
                        from_tree_size: old_sth.sth.tree_size as i64,
                        to_tree_size: new_sth.sth.tree_size as i64,
                        error_msg: &format!("{}", $e)
                      };
                      diesel::insert_into(cfe::cert_fetch_errors)
                          .values(&ins)
                          .execute(db).unwrap_or_display_err();
                      has_error = true;
                    };
                }
                macro_rules! cfe_try {
                    ($r:expr) => {
                      match $r {
                        Ok(k) => k,
                        Err(e) => {
                          cfe_insert!(e);
                          break 'o;
                        }
                      }
                    };
                }
                let mut leid = old_sth.sth.tree_size;
                for le in ctclient::internal::get_entries(&http_client, &parsed_url, old_sth.sth.tree_size..new_sth.sth.tree_size) {
                  let le = cfe_try!(le);
                  leaf_hashs.push(le.hash);
                  if let Err(e) = check_cert(db, log.log_id, &le, leid) {
                    cfe_insert!(format!("Certificate error (leaf #{}={}): {}", leid, ctclient::utils::u8_to_hex(&le.hash), e));
                  }
                  leid += 1;
                }
                assert_eq!(leaf_hashs.len(), (new_sth.sth.tree_size - old_sth.sth.tree_size) as usize);
                for proof_part in consistency_proof_parts {
                  assert!(proof_part.subtree.0 >= old_sth.sth.tree_size);
                  assert!(proof_part.subtree.1 <= new_sth.sth.tree_size);
                  if let Err(mut e) = proof_part.verify(&leaf_hashs[(proof_part.subtree.0 - old_sth.sth.tree_size) as usize..(proof_part.subtree.1 - old_sth.sth.tree_size) as usize]) {
                    e.insert_str(0, "Fetched leaf does not match consistency proof: ");
                    cfe_insert!(e);
                    break 'o;
                  }
                }
                if !has_error {
                  advance_latest_sth(db, &new_sth);
                  diesel::delete(cfe::cert_fetch_errors)
                      .filter(
                        cfe::from_tree_size.eq(old_sth.sth.tree_size as i64)
                            .and(cfe::to_tree_size.eq(new_sth.sth.tree_size as i64))
                      ).execute(db).unwrap_or_display_err();
                  last_fetched_sth = Some(new_sth);
                }
              }
            }
          }
        }

        'o: for &it in &[false, true] {
          let sleep_time = match it {
            false => 250,
            true => 4750
          };
          match recv.recv_timeout(Duration::from_millis(sleep_time)) {
            Err(mpsc::RecvTimeoutError::Timeout) => {
              current_db_hdl = None;
            }
            r @ Err(_) => { r.unwrap(); }
            Ok(msg) => {
              match msg {
                ChannelMessage::Stop => {
                  return;
                }
              }
              break 'o;
            }
          }
        }
      }
    })) {
      Ok(()) => {}
      Err(_) => {
        std::process::abort();
      }
    }
  }).unwrap();
  Handle { jh: MaybeUninit::new(jh), sender }
}

fn check_cert(db: &DBPooledConn, logid: Hash, leaf: &Leaf, leaf_index: u64) -> Result<(), String> {
  let chain = leaf.verify_and_get_x509_chain().map_err(|e| format!("{}", e))?;
  db.build_transaction().read_committed().run(|| -> Result<(), diesel::result::Error> {
    let fp = crate::models::inserts::insert_x509_and_chain(db, &chain)?;
    use crate::schema::certificate_appears_in_leaf::dsl::certificate_appears_in_leaf;
    diesel::insert_into(certificate_appears_in_leaf)
        .values(crate::models::inserts::CertificateAppearsInLeaf {
          leaf_hash: Hash(leaf.hash),
          cert_fp: fp,
          log_id: logid,
          leaf_index: leaf_index as i64
        })
        .on_conflict_do_nothing()
        .execute(db)?;
    Ok(())
  }).unwrap_or_display_err();
  Ok(())
}
