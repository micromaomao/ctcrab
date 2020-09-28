use std::convert::TryInto;
use std::error::Error;

use chrono::{DateTime, Utc};
use diesel::expression::count::count_star;
use diesel::prelude::*;
use rocket::{Request, Response, State};
use rocket::http::Status;
use rocket::response::Responder;
use rocket_contrib::json::Json;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::core::context::CtCrabContext;
use crate::core::db::DBPooledConn;
use crate::models::Hash;

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
  fn respond_to(self, request: &Request) -> rocket::response::Result<'static> {
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
#[derive(Debug, Error)]
#[error("{0} not found.")]
struct NotFound(&'static str);

pub type CtLogs = Vec<BasicCtLogInfo>;
#[derive(Serialize)]
pub struct BasicCtLogInfo {
  log_id: Hash,
  name: String,
  endpoint_url: String,
  latest_sth: Option<BasicSthInfo>,
  last_sth_error: Option<String>
}
#[derive(Serialize)]
pub struct BasicSthInfo {
  id: i64,
  tree_size: u64,
  tree_hash: Hash,
  received_time: TimestampMs,
  sth_timestamp: i64
}

fn get_basic_sth_info(sth_id: Option<i64>, db: &DBPooledConn) -> Result<Option<BasicSthInfo>, Box<dyn Error>> {
  if let Some(sth_id) = sth_id {
    use crate::schema::sth::dsl::*;
    let mut res: Vec<(i64, DateTime<Utc>, i64, Hash, i64)> = sth
        .select((id, received_time, tree_size, tree_hash, sth_timestamp))
        .filter(id.eq(sth_id))
        .load(db)?;
    if res.len() != 1 {
      return Err(Box::new(ExactlyOneExpected("sth")));
    }
    let res = res.swap_remove(0);
    Ok(Some(BasicSthInfo {
      id: res.0,
      tree_size: res.2 as u64,
      tree_hash: res.3,
      received_time: TimestampMs(res.1),
      sth_timestamp: res.4
    }))
  } else {
    Ok(None)
  }
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
  Ok(Json(logs.into_iter().map(|log| -> Result<BasicCtLogInfo, Box<dyn Error>> {
    let lsth = get_basic_sth_info(log.3, &db)?;
    Ok(BasicCtLogInfo {
      log_id: log.0,
      name: log.1,
      endpoint_url: log.2,
      latest_sth: lsth,
      last_sth_error: log.4
    })
  }).collect::<Result<Vec<BasicCtLogInfo>, Box<dyn Error>>>()?))
}

#[get("/log/<id>")]
pub fn log(id: Hash, ctx: State<CtCrabContext>) -> Result<Json<crate::models::CtLog>, APIError> {
  use crate::schema::ctlogs::dsl::*;
  let res: Vec<crate::models::CtLog> = ctlogs
      .filter(log_id.eq(id))
      .load(&ctx.db()?).map_err(|e| Box::new(e) as Box<dyn Error>)?;
  if let Some(res) = res.into_iter().next() {
    Ok(Json(res))
  } else {
    Err(APIError(404, Box::new(NotFound("log"))))
  }
}

#[derive(Debug, Serialize)]
pub struct Stats {
  nb_logs_active: usize,
  nb_logs_total: usize
}

#[get("/stats")]
pub fn stats(ctx: State<CtCrabContext>) -> Result<Json<Stats>, APIError> {
  use crate::schema::ctlogs::dsl::*;
  let db = ctx.db()?;
  let nb_logs_active: i64 = ctlogs.select(count_star()).filter(monitoring.eq(true)).first(&db)
      .map_err(|e| Box::new(e) as Box<dyn Error>)?;
  let nb_logs_total: i64 = ctlogs.select(count_star()).first(&db)
      .map_err(|e| Box::new(e) as Box<dyn Error>)?;
  Ok(Json(Stats {
    nb_logs_active: nb_logs_active.try_into().unwrap(), nb_logs_total: nb_logs_total.try_into().unwrap()
  }))
}

pub fn api_routes() -> Vec<rocket::Route> {
  routes![ctlogs, log, stats]
}
