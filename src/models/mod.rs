use std::time::SystemTime;

use serde::Serialize;

pub use bytea_t::*;

use crate::schema::*;

mod bytea_t;
pub mod inserts;

#[derive(Queryable, QueryableByName, Serialize, Debug)]
#[table_name = "ctlogs"]
pub struct CtLog {
  pub log_id: Hash,
  pub endpoint_url: String,
  pub name: String,
  pub public_key: BytesWithBase64Repr,
  pub monitoring: bool,
  pub latest_tree_hash: Option<Hash>,
  pub latest_tree_size: Option<i64>,
  pub backward_tree_hash: Option<Hash>,
  pub backward_tree_size: Option<i64>,
}

#[derive(Queryable, QueryableByName, Serialize, Debug)]
#[table_name = "sth"]
pub struct Sth {
  pub id: i64,
  pub log_id: Hash,
  pub tree_hash: Hash,
  pub tree_size: i64,
  pub sth_timestamp: i64,
  pub received_time: SystemTime,
  pub signature: BytesWithBase64Repr,
  pub consistent_with_latest: bool,
}

#[test]
fn log_serialisation() {
  let stuff = CtLog { log_id: Hash([173, 247, 190, 250, 124, 255, 16, 200, 139, 157, 61, 156, 30, 62, 24, 106, 180, 103, 41, 93, 207, 177, 12, 36, 202, 133, 134, 52, 235, 220, 130, 138]), endpoint_url: "https://ct.googleapis.com/logs/xenon2023/".to_owned(), name: "Xenon 2023".to_owned(), public_key: BytesWithBase64Repr(vec![48, 89, 48, 19, 6, 7, 42, 134, 72, 206, 61, 2, 1, 6, 8, 42, 134, 72, 206, 61, 3, 1, 7, 3, 66, 0, 4, 114, 22, 62, 11, 239, 239, 206, 62, 96, 221, 149, 203, 99, 122, 185, 169, 141, 74, 111, 108, 220, 97, 128, 166, 69, 94, 47, 131, 172, 148, 243, 133, 136, 208, 165, 116, 208, 123, 142, 255, 197, 238, 66, 162, 240, 45, 147, 227, 194, 208, 178, 153, 226, 225, 66, 233, 210, 198, 0, 39, 105, 116, 174, 206]), monitoring: true, latest_tree_hash: None, latest_tree_size: None, backward_tree_hash: None, backward_tree_size: None };
  let expected_json = r#"{"log_id":"adf7befa7cff10c88b9d3d9c1e3e186ab467295dcfb10c24ca858634ebdc828a","endpoint_url":"https://ct.googleapis.com/logs/xenon2023/","name":"Xenon 2023","public_key":"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEchY+C+/vzj5g3ZXLY3q5qY1Kb2zcYYCmRV4vg6yU84WI0KV00HuO/8XuQqLwLZPjwtCymeLhQunSxgAnaXSuzg==","monitoring":true,"latest_tree_hash":null,"latest_tree_size":null,"backward_tree_hash":null,"backward_tree_size":null}"#;
  assert_eq!(serde_json::to_string(&stuff).unwrap(), expected_json);
}

