use diesel::Connection;
use diesel::pg::Pg;
use diesel::pg::types::date_and_time::PgTimestamp;

pub use bytea_t::*;

use crate::schema::*;

mod bytea_t;
pub mod inserts;

#[derive(Queryable, QueryableByName, Debug)]
#[table_name = "ctlogs"]
pub struct CtLog {
  pub log_id: Hash,
  pub endpoint_url: String,
  pub name: String,
  pub public_key: BytesWithBase64Repr,
  pub monitoring: bool,
  pub latest_sth: Option<i64>,
  pub last_sth_error: Option<String>
}

impl CtLog {
  fn get_latest_sth<C: Connection<Backend = Pg>>(&self, db: C) -> Result<Option<Sth>, diesel::result::Error> {
    use diesel::prelude::*;
    match self.latest_sth {
      Some(sth_id) => {
        let mut sths: Vec<Sth> = sth::dsl::sth.filter(sth::dsl::id.eq(sth_id)).load(&db)?;
        Ok(Some(sths.swap_remove(0)))
      },
      None => Ok(None)
    }
  }
}

#[derive(Queryable, QueryableByName, Debug)]
#[table_name = "sth"]
pub struct Sth {
  pub id: i64,
  pub log_id: Hash,
  pub tree_hash: Hash,
  pub tree_size: i64,
  pub sth_timestamp: i64,
  pub received_time: PgTimestamp,
  pub signature: BytesWithBase64Repr,
  pub checked_consistent_with_latest: bool
}

