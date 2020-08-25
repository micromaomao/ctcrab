use std::mem::{MaybeUninit, replace};
use std::panic::AssertUnwindSafe;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use ctclient::{CTClient, SignedTreeHead, SthResult};
use thiserror::Error;

use crate::core::db::{DBPool, PgConnectionHelper};
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
      trait Helper {
        type Inner;
        fn unwrap_or_display_err(self) -> Self::Inner;
      }
      impl<T> Helper for Result<T, diesel::result::Error> {
        type Inner = T;

        fn unwrap_or_display_err(self) -> T {
          match self {
            Ok(t) => t,
            Err(e) => panic!("Database error: {}", e)
          }
        }
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
      let fetch_sth = || -> Result<FetchedSth, FetchSthError> {
        let th = ctclient::internal::check_tree_head(
          &http_client,
          &parsed_url,
          &parsed_pub_key,
        )?;
        if th.tree_size > i64::MAX as u64 {
          return Err(FetchSthError::TreeSizeTooLarge);
        }
        let db = get_db!();
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
              .get_results(&db)?;
          if res.is_empty() {
            // already exists
            let existing_id: Vec<i64> = sth.select(sth_id)
                .filter(
                  sth_log_id.eq(&log.log_id)
                      .and(tree_size.eq(ins.tree_size))
                      .and(tree_hash.eq(ins.tree_hash))
                      .and(sth_timestamp.eq(ins.sth_timestamp)))
                .load(&db)?;
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

      let advance_latest_sth = |new_latest: &FetchedSth| {
        let db = get_db!();
        db.transaction_rw_serializable::<(), diesel::result::Error, _>(|| {
          diesel::update(sth)
              .filter(sth_id.eq(new_latest.stored_as_id))
              .set(checked_consistent_with_latest.eq(true))
              .execute(&db)?;
          diesel::update(ctlogs)
              .filter(ctlogs_log_id.eq(&log.log_id))
              .set(latest_sth.eq(new_latest.stored_as_id))
              .execute(&db)?;
          Ok(())
        }).unwrap_or_display_err();

        let sth_to_check: Vec<Sth> =
            sth.filter(
              checked_consistent_with_latest.eq(false)
                  .and(sth_log_id.eq(&log.log_id))
                  .and(tree_size.le(new_latest.sth.tree_size as i64))
            ).load(&db).unwrap_or_display_err();
        for _s in sth_to_check {
          unimplemented!();
        }
      };

      let mut last_fetched_sth: Option<FetchedSth> = None;
      if let Some(latest_sth_id) = log.latest_sth {
        let db = get_db!();
        let mut stored_sth: Vec<Sth> = sth
            .filter(sth_id.eq(latest_sth_id))
            .load(&db).unwrap_or_display_err();
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
        let new_sth = match fetch_sth() {
          Ok(s) => {
            let db = get_db!();
            diesel::update(ctlogs)
                .filter(ctlogs_log_id.eq(&log.log_id))
                .set(last_sth_error.eq(None::<String>))
                .execute(&db).unwrap_or_display_err();
            s
          },
          Err(e) => {
            let db = get_db!();
            diesel::update(ctlogs)
                .filter(ctlogs_log_id.eq(&log.log_id))
                .set(last_sth_error.eq(format!("{}", e)))
                .execute(&db).unwrap_or_display_err();
            continue;
          }
        };
        match last_fetched_sth {
          None => {
            advance_latest_sth(&new_sth);
            last_fetched_sth = Some(new_sth);
          },
          Some(_old_sth) => {
            unimplemented!()
          }
        }

        match recv.recv_timeout(Duration::from_secs(5)) {
          Err(mpsc::RecvTimeoutError::Timeout) => {}
          r @ Err(_) => { r.unwrap(); }
          Ok(msg) => {
            match msg {
              ChannelMessage::Stop => {
                return;
              }
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

