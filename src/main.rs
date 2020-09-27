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
use rocket::http::{Status, Header};
use rocket::response::content::Plain;
use rocket::response::Responder;
use serde::Serialize;
use serde_json::json;

use crate::api::APIError;
use crate::core::context::CtCrabContext;
use crate::models::Hash;
use chrono::{DateTime, Utc, NaiveDateTime};
use rocket::fairing::{Fairing, Info};

mod schema;
mod models;
mod core;
mod api;

#[catch(500)]
fn http500catcher() -> api::APIError {
  #[derive(Debug, Error)]
  #[error("Whoops! Looks like we messed up.")]
  struct E;
  api::APIError(500, Box::new(E))
}

#[catch(404)]
fn http404catcher(req: &Request) -> api::APIError {
  #[derive(Debug, Error)]
  #[error("Page not found")]
  struct PNF;
  api::APIError(404, Box::new(PNF))
}

struct AccessControlFairing;

impl Fairing for AccessControlFairing {
  fn info(&self) -> Info {
    Info {
      name: "Access control allow origin: *",
      kind: rocket::fairing::Kind::Response
    }
  }

  fn on_response(&self, request: &Request, response: &mut Response) {
    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
  }
}

fn main() {
  let ctx = match CtCrabContext::new() {
    Ok(ctx) => ctx,
    Err(e) => panic!("{}", e)
  };
  rocket::ignite()
      .mount("/", api::api_routes())
      .register(catchers![http500catcher, http404catcher])
      .attach(AccessControlFairing)
      .manage(ctx).launch();
}
