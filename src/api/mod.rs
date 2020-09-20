use std::error::Error;
use std::time::SystemTime;

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

pub struct TimestampMs(PgTimestamp);
impl Serialize for TimestampMs {
  fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
      S: Serializer {
    serializer.serialize_i64((self.0).0 / 1000 + 946684800000)
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
  received_time: TimestampMs,
  tree_size: u64,
  tree_hash: Hash
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
      let mut res: Vec<(i64, PgTimestamp, i64, Hash)> = sth
          .select((id, received_time, tree_size, tree_hash))
          .filter(id.eq(latest_sth_id))
          .load(&db)?;
      if res.len() != 1 {
        return Err(Box::new(ExactlyOneExpected("sth")));
      }
      let res = res.swap_remove(0);
      Some(LogLatestSth {
        id: res.0,
        received_time: TimestampMs(res.1),
        tree_size: res.2 as u64,
        tree_hash: res.3
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

pub fn api_routes() -> Vec<rocket::Route> {
  routes![ctlogs]
}
