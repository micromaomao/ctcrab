use crate::core::db::{DBPooledConn, PgConnectionHelper};
use crate::models::Hash;
use diesel::prelude::*;

#[derive(Debug, Error)]
pub enum E {
  #[error("Unable to fetch log list")]
  UnableToFetchLogList,
  #[error("DB: {0}")]
  DB(#[source] diesel::result::Error),
  #[error("{log_id}'s public key changed, which is not allowed.")]
  PublicKeyChanged { log_id: Hash, name: String }
}
impl From<diesel::result::Error> for E {
  fn from(e: diesel::result::Error) -> Self {
    E::DB(e)
  }
}

pub fn initialise_or_update_ctlogs_table(db: &DBPooledConn) -> Result<(), E> {
  let ll = ctclient::google_log_list::LogList::get().map_err(|_| E::UnableToFetchLogList)?;
  use crate::schema::ctlogs::dsl::*;
  for (id, log) in ll.map_id_to_log {
    use ctclient::google_log_list::LogState::*;
    match log.state {
      Pending | Qualified | Usable | Readonly => {
        let ins = crate::models::inserts::CtLog {
          log_id: Hash(id),
          endpoint_url: &log.base_url,
          name: &log.description,
          public_key: &log.pub_key,
          monitoring: true,
        };
        db.transaction_rw_serializable(|| {
          let existing: Vec<(Vec<u8>,)> = ctlogs
              .select((public_key,))
              .filter(log_id.eq(ins.log_id))
              .limit(1)
              .load(db)?;
          if existing.is_empty() {
            diesel::insert_into(ctlogs)
                .values(&ins)
                .execute(db)?;
          } else {
            let pub_key = &existing[0].0[..];
            if pub_key != ins.public_key {
              return Err(E::PublicKeyChanged {log_id: ins.log_id, name: ins.name.to_owned()});
            }
            diesel::update(ctlogs)
                .filter(log_id.eq(&ins.log_id))
                .set((
                  endpoint_url.eq(&ins.endpoint_url),
                  name.eq(&ins.name),
                  monitoring.eq(true)
                )).execute(db)?;
          }
          Ok(())
        })?;
      }
      Retired | Rejected => {
        diesel::update(ctlogs)
            .filter(log_id.eq(Hash(id)))
            .set(monitoring.eq(false))
            .execute(db)?;
      }
    }
  }
  Ok(())
}
