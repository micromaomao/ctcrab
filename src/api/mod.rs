use std::error::Error;
use std::time::{SystemTime, Duration};

use diesel::pg::types::date_and_time::PgTimestamp;
use diesel::prelude::*;
use rocket::response::{Debug, Responder};
use rocket::{State, Request, Response};
use rocket_contrib::json::Json;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::core::context::CtCrabContext;
use crate::models::Hash;
use rocket::http::Status;
use ctclient::internal::re_exports::reqwest::Url;
use std::ops::Add;
use std::convert::TryFrom;
use chrono::{DateTime, Utc, NaiveDateTime, SecondsFormat};

pub struct TimestampMs(DateTime<Utc>);
impl Serialize for TimestampMs {
  fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
      S: Serializer {
    serializer.serialize_i64(self.0.timestamp_millis())
  }
}

#[derive(Debug)]
pub struct APIError(pub u16, pub Box<dyn Error>);
impl<'r> Responder<'r> for APIError {
  fn respond_to(self, request: &Request) -> rocket::response::Result<'r> {
    let res = Response::build_from(rocket::response::content::Plain(format!("{}", &self.1)).respond_to(request)?)
        .status(Status::from_code(self.0).unwrap()).finalize();
    Ok(res)
  }
}
impl From<Box<dyn Error>> for APIError {
  fn from(e: Box<dyn Error>) -> Self {
    #[derive(Debug, Error)]
    #[error("Whoops! Looks like we messed up. (Unknown internal error)")]
    struct Wrapper(#[source] Box<dyn Error>);
    APIError(500, Box::new(Wrapper(e)) as Box<_>)
  }
}

#[derive(Debug, Error)]
#[error("Expected to find exactly one {0}.")]
struct ExactlyOneExpected(&'static str);
#[derive(Debug, Error)]
#[error("Tried to process a negative duration.")]
struct SignedDurationIsNegative(#[source] Option<Box<dyn Error>>);

pub type CtLogs = Vec<CtLog>;
#[derive(Serialize)]
pub struct CtLog {
  log_id: Hash,
  name: String,
  endpoint_url: String,
  latest_sth: Option<LogLatestSth>,
  last_sth_error: Option<String>
}
#[derive(Serialize)]
pub struct LogLatestSth {
  id: i64,
  tree_size: u64,
  tree_hash: Hash,
  latency_str: String,
  latency_based_on: String,
  received_time: TimestampMs,
  sth_timestamp: i64,
  latency_more_than_24h: bool
}

#[get("/ctlogs")]
pub fn ctlogs(ctx: State<CtCrabContext>) -> Result<Json<CtLogs>, APIError> {
  let db = ctx.db()?;
  use crate::schema::ctlogs::dsl::*;
  let logs: Vec<(Hash, String, String, Option<i64>, Option<String>)> = ctlogs
      .select((log_id, name, endpoint_url, latest_sth, last_sth_error))
      .filter(monitoring.eq(true))
      .order_by(name.asc())
      .load(&db).map_err(|e| Box::new(e) as Box<dyn Error>)?;
  Ok(Json(logs.into_iter().map(|log| -> Result<CtLog, Box<dyn Error>> {
    let lsth = if let Some(latest_sth_id) = log.3 {
      use crate::schema::sth::dsl::*;
      let mut res: Vec<(i64, DateTime<Utc>, i64, Hash, i64)> = sth
          .select((id, received_time, tree_size, tree_hash, sth_timestamp))
          .filter(id.eq(latest_sth_id))
          .load(&db)?;
      if res.len() != 1 {
        return Err(Box::new(ExactlyOneExpected("sth")));
      }
      let res = res.swap_remove(0);
      let mut sth_time = DateTime::from_utc(NaiveDateTime::from_timestamp(res.4 / 1000, (res.4 % 1000) as u32 * 1000000), Utc);
      let received = res.1;
      if sth_time > received {
        sth_time = received;
      }
      let latency = std::time::Duration::from_secs(u64::try_from(Utc::now()
          .signed_duration_since(sth_time)
          .num_seconds()).map_err(|e| Box::new(SignedDurationIsNegative(None)) as Box<_>)?);
      Some(LogLatestSth {
        id: res.0,
        tree_size: res.2 as u64,
        tree_hash: res.3,
        latency_str: humantime::format_duration(latency).to_string(),
        latency_based_on: sth_time.to_rfc3339_opts(SecondsFormat::Secs, true),
        received_time: TimestampMs(received),
        sth_timestamp: res.4,
        latency_more_than_24h: latency.as_secs() >= 60*60*24
      })
    } else {
      None
    };
    Ok(CtLog {
      log_id: log.0,
      name: log.1,
      endpoint_url: log.2,
      latest_sth: lsth,
      last_sth_error: log.4
    })
  }).collect::<Result<Vec<CtLog>, Box<dyn Error>>>()?))
}

#[derive(Serialize)]
pub struct CtLogDetail {
  log_id: Hash,
  endpoint_url: String,
  name: String,
  monitoring: bool,
  last_sth_error: String
}

#[get("/log/<id>")]
pub fn log(id: Hash, ctx: State<CtCrabContext>) -> Result<Json<CtLogDetail>, APIError> {
  unimplemented!()
}

pub fn api_routes() -> Vec<rocket::Route> {
  routes![ctlogs, log]
}
