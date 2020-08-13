#[macro_use]
extern crate diesel;

mod schema;
mod models;
mod core;
use crate::core::open_db;
use diesel::RunQueryDsl;

fn main() -> Result<(), &'static str> {
  use std::convert::TryInto;
  use models::{Hash, BytesWithBase64Repr};
  use schema::ctlogs::dsl::*;
  use diesel::RunQueryDsl;

  let db = open_db();
  let ll = ctclient::google_log_list::LogList::get().map_err(|_| "Unable to fetch log list")?;
  diesel::delete(ctlogs).execute(&db).map_err(|_| "SQL error")?;
  for (id, log) in ll.map_id_to_log {
    if log.state != ctclient::google_log_list::LogState::Usable {
      continue;
    }
    let ins = models::CtLog {
      log_id: Hash(id[..].try_into().unwrap()),
      endpoint_url: log.base_url.clone(),
      name: log.description.clone(),
      public_key: BytesWithBase64Repr(log.pub_key.clone())
    };
    diesel::insert_into(ctlogs).values(ins).execute(&db).map_err(|_| "SQL error")?;
  }
  Ok(())
}
