#![feature(proc_macro_hygiene, decl_macro)]
#![feature(label_break_value)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate thiserror;

use std::error::Error;

use rocket::{Request, Response, State};
use rocket::response::content::Plain;
use rocket::response::Responder;
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::core::context::CtCrabContext;
use rocket::http::Status;
use crate::api::APIError;

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
    #[derive(Serialize)]
    struct Ctx<'a> {
      main_error: &'a str,
      cause_chain: &'a [String],
      status: u16,
      version: &'static str
    }
    let ctx = Ctx {
      status,
      main_error: &error_chain[0],
      cause_chain: &error_chain[1..],
      version: env!("CARGO_PKG_VERSION")
    };
    let res = Response::build_from(
      Template::render("error", ctx).respond_to(request)?
    ).status(Status::from_code(status).unwrap()).finalize();
    Ok(res)
  }
}
impl From<api::APIError> for HTMLError {
  fn from(e: APIError) -> Self {
    HTMLError(e.0, e.1)
  }
}

#[get("/")]
fn index(ctx: State<CtCrabContext>) -> Result<Template, HTMLError> {
  #[derive(Serialize)]
  struct Ctx {
    ctlogs: api::CtLogs,
    nb_logs: usize
  }
  let ctlogs = api::ctlogs(ctx)?.0;
  Ok(Template::render("index", Ctx {
    nb_logs: ctlogs.len(),
    ctlogs
  }))
}

#[catch(500)]
fn http500catcher() -> Plain<&'static str> {
  Plain("Whoops! Looks like we messed up. (HTTP 500)\n")
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
      .mount("/", routes![ index ])
      .mount("/api/", api::api_routes())
      .register(catchers![http500catcher, http404catcher])
      .manage(ctx).launch();
}
