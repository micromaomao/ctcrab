#![feature(label_break_value)]
#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::RunQueryDsl;

use models::Hash;

use crate::core::db::open_db;
use crate::core::db::PgConnectionHelper;

mod schema;
mod models;
mod core;

#[derive(Debug)]
enum E {
  UnableToFetchLogList,
  DB(diesel::result::Error),
  PublicKeyChanged { log_id: Hash, name: String }
}
impl From<diesel::result::Error> for E {
  fn from(e: diesel::result::Error) -> Self {
    E::DB(e)
  }
}
impl From<E> for String {
  fn from(e: E) -> Self {
    match e {
      E::UnableToFetchLogList => "Unable to fetch log list".to_owned(),
      E::DB(e) => format!("{}", e),
      E::PublicKeyChanged { log_id: _, name } => format!("{:?}'s public key changed, which is not allowed.", name)
    }
  }
}

fn main() -> Result<(), E> {
  let db = open_db();
  let ll = ctclient::google_log_list::LogList::get().map_err(|_| E::UnableToFetchLogList)?;
  use schema::ctlogs::dsl::*;
  for (id, log) in ll.map_id_to_log {
    use ctclient::google_log_list::LogState::*;
    match log.state {
      Pending | Qualified | Usable | Readonly => {
        let ins = models::inserts::CtLog {
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
              .load(&db)?;
          if existing.is_empty() {
            diesel::insert_into(ctlogs)
                .values(&ins)
                .execute(&db)?;
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
                )).execute(&db)?;
          }
          Ok(())
        })?;
      }
      Retired | Rejected => {
        diesel::update(ctlogs)
            .filter(log_id.eq(Hash(id)))
            .set(monitoring.eq(false))
            .execute(&db)?;
      }
    }
  }
  Ok(())
}
