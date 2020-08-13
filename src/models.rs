use std::convert::TryInto;
use diesel::prelude::*;
use crate::schema::*;
use diesel::serialize::{ToSql, Output};
use diesel::expression::AsExpression;
use diesel::backend::Backend;
use std::io::Write;
use serde::{Serialize, Deserialize};
use serde::ser::Serializer;
use serde::de::Deserializer;
use serde::export::Formatter;
use diesel::sql_types::Binary;
use diesel::deserialize::FromSql;

macro_rules! impl_sql_binary_type {
  ($type_name:ident) => {
    impl<DB: Backend> ToSql<Binary, DB> for $type_name {
      fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> diesel::serialize::Result {
        ToSql::<Binary, DB>::to_sql(&self.0[..], out)
      }
    }
    impl<DB: Backend<RawValue = [u8]>> FromSql<Binary, DB> for $type_name {
      fn from_sql(bytes: Option<&<DB as Backend>::RawValue>) -> diesel::deserialize::Result<Self> {
        let vv: Vec<u8> = FromSql::<Binary, DB>::from_sql(bytes)?;
        Ok(Self(vv[..].try_into().map_err(|e| Box::new(e))?))
      }
    }
  };
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, AsExpression)]
#[sql_type="Binary"]
pub struct Hash(pub [u8; 32]);
impl_sql_binary_type!(Hash);
impl Serialize for Hash {
  fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
      S: Serializer {
    serializer.serialize_str(&ctclient::utils::u8_to_hex(&self.0))
  }
}
impl<'de> Deserialize<'de> for Hash {
  fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
      D: Deserializer<'de> {
    struct MyVisitor;
    impl<'de> serde::de::Visitor<'de> for MyVisitor {
      type Value = Hash;

      fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a 32 byte string in hex")
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where
          E: serde::de::Error, {
        if v.len() != 64 {
          return Err(E::invalid_length(64,&self));
        }
        let mut buf = [0u8; 32];
        let hex = v.as_bytes();
        for i in 0..32 {
          for ii in 0..2 {
            let mut hex = hex[i*2+ii];
            if (b'A'..=b'F').contains(&hex) {
              hex = hex - b'A' + b'a';
            }
            let halfbyte = if (b'0'..b'9').contains(&hex) {
              hex - b'0'
            } else if (b'a'..b'f').contains(&hex) {
              hex - b'a' + 10u8
            } else {
              return Err(E::invalid_value(serde::de::Unexpected::Char(hex as char), &self))
            };
            buf[i] |= halfbyte << (ii * 4);
          }
        }
        Ok(Hash(buf))
      }
    }
    deserializer.deserialize_str(MyVisitor)
  }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, AsExpression)]
#[sql_type="Binary"]
pub struct BytesWithBase64Repr(pub Vec<u8>);
impl_sql_binary_type!(BytesWithBase64Repr);
impl Serialize for BytesWithBase64Repr {
  fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
      S: Serializer {
    serializer.serialize_str(&base64::encode(&self.0))
  }
}
impl<'de> Deserialize<'de> for BytesWithBase64Repr {
  fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
      D: Deserializer<'de> {
    struct MyVisitor;
    impl<'de> serde::de::Visitor<'de> for MyVisitor {
      type Value = BytesWithBase64Repr;

      fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "base64 string")
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where
          E: serde::de::Error, {
        Ok(BytesWithBase64Repr(base64::decode(v).map_err(|e| E::custom(e.to_string()))?))
      }
    }
    deserializer.deserialize_str(MyVisitor)
  }
}

#[derive(Queryable, Insertable, Serialize, Debug)]
#[table_name = "ctlogs"]
pub struct CtLog {
  pub log_id: Hash,
  pub endpoint_url: String,
  pub name: String,
  pub public_key: BytesWithBase64Repr,
}

#[derive(Queryable, Insertable, Serialize, Debug)]
#[table_name = "sth"]
pub struct Sth {
  pub tree_hash: Hash,
  pub log_id: Hash,
  pub sth_timestamp: i64,
  pub tree_size: i64,
  pub signature: BytesWithBase64Repr,
}

#[test]
fn log_serialisation() {
  let stuff = CtLog { log_id: Hash([173, 247, 190, 250, 124, 255, 16, 200, 139, 157, 61, 156, 30, 62, 24, 106, 180, 103, 41, 93, 207, 177, 12, 36, 202, 133, 134, 52, 235, 220, 130, 138]), endpoint_url: "https://ct.googleapis.com/logs/xenon2023/".to_owned(), name: "Xenon 2023".to_owned(), public_key: BytesWithBase64Repr(vec![48, 89, 48, 19, 6, 7, 42, 134, 72, 206, 61, 2, 1, 6, 8, 42, 134, 72, 206, 61, 3, 1, 7, 3, 66, 0, 4, 114, 22, 62, 11, 239, 239, 206, 62, 96, 221, 149, 203, 99, 122, 185, 169, 141, 74, 111, 108, 220, 97, 128, 166, 69, 94, 47, 131, 172, 148, 243, 133, 136, 208, 165, 116, 208, 123, 142, 255, 197, 238, 66, 162, 240, 45, 147, 227, 194, 208, 178, 153, 226, 225, 66, 233, 210, 198, 0, 39, 105, 116, 174, 206]) };
  let expected_json = r#"{"log_id":"adf7befa7cff10c88b9d3d9c1e3e186ab467295dcfb10c24ca858634ebdc828a","endpoint_url":"https://ct.googleapis.com/logs/xenon2023/","name":"Xenon 2023","public_key":"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEchY+C+/vzj5g3ZXLY3q5qY1Kb2zcYYCmRV4vg6yU84WI0KV00HuO/8XuQqLwLZPjwtCymeLhQunSxgAnaXSuzg=="}"#;
  assert_eq!(serde_json::to_string(&stuff).unwrap(), expected_json);
}
