use super::*;
use diesel::expression::functions::date_and_time::now;
use diesel::RunQueryDsl;

#[derive(Insertable, Debug)]
#[table_name = "ctlogs"]
pub struct CtLog<'a> {
  pub log_id: Hash,
  pub endpoint_url: &'a str,
  pub name: &'a str,
  pub public_key: &'a [u8],
  pub monitoring: bool
}

#[derive(Insertable, Debug)]
#[table_name = "sth"]
pub struct Sth<'a> {
  pub log_id: Hash,
  pub tree_hash: Hash,
  pub tree_size: i64,
  pub sth_timestamp: i64,
  pub signature: &'a [u8],
  pub checked_consistent_with_latest: bool
}

#[derive(Insertable, Debug)]
#[table_name = "consistency_check_errors"]
pub struct ConsistencyCheckError<'a> {
  log_id: Hash,
  from_sth_id: i64,
  to_sth_id: i64,
  last_check_error: &'a str
}

impl<'a> ConsistencyCheckError<'a> {
  pub fn upsert<DB: diesel::Connection<Backend = diesel::pg::Pg>>(db: &DB, log_id: Hash, from_sth_id: i64, to_sth_id: i64, last_check_error: &str) -> Result<(), diesel::result::Error> {
    use diesel::prelude::*;
    use crate::schema::consistency_check_errors::dsl;
    diesel::insert_into(dsl::consistency_check_errors)
        .values(ConsistencyCheckError {
          log_id, from_sth_id, to_sth_id, last_check_error
        })
        .on_conflict((dsl::log_id, dsl::to_sth_id, dsl::from_sth_id))
        .do_update()
        .set((dsl::last_check_time.eq(now), dsl::last_check_error.eq(last_check_error)))
        .execute(db).map(|_| {})
  }
}
