use std::collections::HashSet;
use std::convert::TryInto;
use std::iter::FromIterator;

use ctclient::internal::re_exports::openssl;
use diesel::expression::functions::date_and_time::now;
use diesel::prelude::*;
use openssl::hash::MessageDigest;
use openssl::x509::X509;

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

#[derive(Insertable, Debug)]
#[table_name = "cert_fetch_errors"]
pub struct CertFetchError<'a> {
  pub log_id: Hash,
  pub from_tree_size: i64,
  pub to_tree_size: i64,
  pub error_msg: &'a str
}

#[derive(Insertable, Debug)]
#[table_name = "certificates"]
struct Certificate<'a> {
  pub fingerprint: Hash,
  pub x509: &'a [u8]
}

/// **This function must be called within a transaction**
/// # Return
///
/// sha256 fingerprint
pub fn insert_x509_and_chain<DB>(db: &DB, x509_chain: &[X509]) -> Result<Hash, diesel::result::Error>
  where DB: diesel::Connection<Backend=diesel::pg::Pg> {
  let fp = x509_chain[0].digest(MessageDigest::sha256()).unwrap();
  let fp = Hash(fp.as_ref().try_into().unwrap());
  let der_chain = x509_chain.iter().map(|x| x.to_der().unwrap()).collect::<Vec<_>>();
  use crate::schema::certificates::dsl as c_dsl;
  let ins = Certificate {
    fingerprint: fp,
    x509: &der_chain[0]
  };
  let already_existed = diesel::insert_into(c_dsl::certificates)
      .values(ins)
      .on_conflict_do_nothing()
      .execute(db)? == 0;
  if already_existed {
    return Ok(fp);
  }
  use crate::schema::certificate_dns_names::dsl as d_dsl;
  let dns_names = ctclient::certutils::get_dns_names(&x509_chain[0]).unwrap();
  let dns_names = HashSet::<String>::from_iter(dns_names).into_iter().collect::<Vec<String>>();
  let d_vals = dns_names.into_iter()
      .map(|x| (d_dsl::dns_name.eq(x), d_dsl::cert_fp.eq(&fp)))
      .collect::<Vec<_>>();
  diesel::insert_into(d_dsl::certificate_dns_names)
      .values(&d_vals)
      .execute(db)?;
  insert_rest_of_the_chain(db, &fp, &der_chain[1..])?;
  Ok(fp)
}

#[derive(Insertable, Debug)]
#[table_name = "certificate_chain"]
struct CertificateChain<'a> {
  pub certificate_fingerprint: &'a [u8],
  pub chain: &'a [Vec<u8>]
}

fn insert_rest_of_the_chain<DB>(db: &DB, fp: &Hash, rest_of_the_chain: &[Vec<u8>]) -> diesel::result::QueryResult<()>
  where DB: diesel::Connection<Backend = diesel::pg::Pg> {
  let ins = CertificateChain {
    certificate_fingerprint: &fp.0,
    chain: rest_of_the_chain
  };
  use crate::schema::certificate_chain::dsl::certificate_chain;
  diesel::insert_into(certificate_chain)
      .values(ins)
      .execute(db).map(|_| ())
}

#[derive(Insertable, Debug)]
#[table_name = "certificate_appears_in_leaf"]
pub struct CertificateAppearsInLeaf {
  pub leaf_hash: Hash,
  pub cert_fp: Hash,
  pub log_id: Hash,
  pub leaf_index: i64
}
