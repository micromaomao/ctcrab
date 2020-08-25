use super::*;

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
