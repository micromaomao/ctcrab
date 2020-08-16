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
