use std::convert::TryInto;
use std::fmt;
use std::fmt::Display;
use std::io::Write;

use ctclient::utils::u8_to_hex;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Binary;
use serde::de::{Deserialize, Deserializer};
use serde::export::Formatter;
use serde::ser::Serializer;
use serde::Serialize;

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

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, FromSqlRow, AsExpression)]
#[sql_type="Binary"]
pub struct Hash(pub [u8; 32]);
impl_sql_binary_type!(Hash);
impl Serialize for Hash {
  fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
      S: Serializer {
    serializer.serialize_str(&u8_to_hex(&self.0))
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

impl Display for Hash {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", u8_to_hex(&self.0))
  }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, FromSqlRow, AsExpression)]
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
