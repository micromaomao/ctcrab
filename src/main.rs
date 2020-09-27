#![feature(proc_macro_hygiene, decl_macro)]
#![feature(label_break_value)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate thiserror;

use std::error::Error;

use diesel::prelude::*;
use rocket::{Request, Response, State};
use rocket::http::Status;
use rocket::response::content::Plain;
use rocket::response::Responder;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use serde::Serialize;
use serde_json::json;

use crate::api::APIError;
use crate::core::context::CtCrabContext;
use crate::models::Hash;
use chrono::{DateTime, Utc, NaiveDateTime};

mod schema;
mod models;
mod core;
mod api;

#[derive(Debug)]
struct HTMLError(u16, Box<dyn Error>);
impl<'r> Responder<'r> for HTMLError {
  fn respond_to(self, request: &Request) -> rocket::response::Result<'r> {
    let status = self.0;
    let mut error_chain = Vec::new();
    let mut current_err = &*self.1;
    loop {
      error_chain.push(format!("{}", current_err));
      if let Some(next_err) = current_err.source() {
        current_err = next_err;
      } else {
        break;
      }
    }
    let res = Response::build_from(
      Template::render("error", json!({
        "status": status,
        "main_error": &error_chain[0],
        "cause_chain": &error_chain[1..],
        "has_cause": error_chain.len() > 1,
        "version": env!("CARGO_PKG_VERSION")
      })).respond_to(request)?
    ).status(Status::from_code(status).unwrap()).finalize();
    Ok(res)
  }
}
impl From<api::APIError> for HTMLError {
  fn from(e: APIError) -> Self {
    HTMLError(e.0, e.1)
  }
}
impl From<Box<dyn Error>> for HTMLError {
  fn from(e: Box<dyn Error>) -> Self {
    HTMLError(500, e)
  }
}

#[get("/")]
fn index(ctx: State<CtCrabContext>) -> Result<Template, HTMLError> {
  let ctlogs = api::ctlogs(ctx)?.0;
  Ok(Template::render("index", json!({
    "nb_logs": ctlogs.len(),
    "nb_domains": 0usize,
    "ctlogs": ctlogs
  })))
}

#[get("/log/<id>")]
fn log(id: Hash, ctx: State<CtCrabContext>) -> Result<Template, HTMLError> {
  let log_detail = api::log(id, ctx)?.0;
  let mut latency_warning: Option<String> = None;
  if let Some(latest_sth) = log_detail.latest_sth {
    use crate::schema::sth::dsl::*;
    use crate::models::Sth;
    let s: Vec<Sth> = sth.filter(id.eq(latest_sth)).load(&ctx.db()?).map_err(|e| Box::new(e) as Box<dyn Error>)?;
    if s.len() != 1 {
      #[derive(Debug, Error)]
      #[error("Expected to find sth with id {0}.")]
      struct E(i64);
      return Err((Box::new(E(latest_sth)) as Box<dyn Error>).into())
    }
    let s = s.into_iter().next().unwrap();
    let s_timestamp = DateTime::<Utc>::from_utc(
      NaiveDateTime::from_timestamp(s.sth_timestamp / 1000, (s.sth_timestamp % 1000) as u32 * 1000000),
      Utc
    );
    let sth_effective_time = s.received_time.min(s_timestamp);
    let lat = Utc::now().signed_duration_since(sth_effective_time);
    if lat.num_days() >= 1 {
      latency_warning = Some(humantime::format_duration(lat.to_std().unwrap()).to_string());
    }
  }
  Ok(Template::render("log", json!({
    "log_detail": log_detail,
    "latency_warning": latency_warning
  })))
}

#[catch(500)]
fn http500catcher() -> HTMLError {
  #[derive(Debug, Error)]
  #[error("Whoops! Looks like we messed up.")]
  struct E;
  HTMLError(500, Box::new(E))
}

#[catch(404)]
fn http404catcher(req: &Request) -> Result<Response<'static>, Status> {
  if req.uri().path().starts_with("/api/") {
    Plain("Unknown endpoint (HTTP 404)\n").respond_to(req)
  } else {
    #[derive(Debug, Error)]
    #[error("Page not found")]
    struct PNF;
    HTMLError(404, Box::new(PNF)).respond_to(req)
  }
}

fn main() {
  let ctx = match CtCrabContext::new() {
    Ok(ctx) => ctx,
    Err(e) => panic!("{}", e)
  };
  rocket::ignite()
      .attach(Template::fairing())
      .mount("/", routes![ index, log ])
      // todo: support http caching
      .mount("/", StaticFiles::new("static", rocket_contrib::serve::Options::None))
      .mount("/api/", api::api_routes())
      .register(catchers![http500catcher, http404catcher])
      .manage(ctx).launch();
}
