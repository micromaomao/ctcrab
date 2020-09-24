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
use serde::export::{Formatter, TryFrom};
use serde::ser::Serializer;
use serde::Serialize;
use std::str::FromStr;
use rocket::request::FromParam;
use rocket::http::RawStr;

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
        Ok(Self(vv[..].try_into().map_err(Box::new)?))
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
#[derive(Debug, Error)]
pub enum HashFromStrError {
  #[error("Expected length of 64 (hex of 32 byte hash), got {0}.")]
  InvalidLength(usize),
  #[error("Unexpected byte {0:?}, expected ASCII 0-9a-f.")]
  UnexpectedByte(u8)
}
impl TryFrom<&[u8]> for Hash {
  type Error = HashFromStrError;

  fn try_from(hex: &[u8]) -> Result<Self, Self::Error> {
    if hex.len() != 64 {
      return Err(HashFromStrError::InvalidLength(hex.len()));
    }
    let mut buf = [0u8; 32];
    for i in 0..32 {
      for ii in 0..2 {
        let mut hex = hex[i*2+ii];
        if (b'A'..=b'F').contains(&hex) {
          hex = hex - b'A' + b'a';
        }
        let halfbyte = if (b'0'..=b'9').contains(&hex) {
          hex - b'0'
        } else if (b'a'..=b'f').contains(&hex) {
          hex - b'a' + 10u8
        } else {
          return Err(HashFromStrError::UnexpectedByte(hex));
        };
        buf[i] |= halfbyte << (ii * 4);
      }
    }
    Ok(Hash(buf))
  }
}
impl FromStr for Hash {
  type Err = HashFromStrError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.as_bytes().try_into()
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
        match Hash::from_str(v) {
          Ok(h) => Ok(h),
          Err(HashFromStrError::InvalidLength(got)) => Err(E::invalid_length(64,&self)),
          Err(HashFromStrError::UnexpectedByte(b)) => Err(E::invalid_value(serde::de::Unexpected::Char(b as char), &self))
        }
      }
    }
    deserializer.deserialize_str(MyVisitor)
  }
}
impl<'a> FromParam<'a> for Hash {
  type Error = HashFromStrError;

  fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
    let s = param.as_bytes();
    s.try_into()
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
