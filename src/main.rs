#![feature(proc_macro_hygiene, decl_macro)]
#![feature(label_break_value)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate thiserror;

use std::convert::TryFrom;
use std::error::Error;
use std::time::Duration;

use diesel::expression::count::count_star;
use diesel::prelude::*;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info};
use rocket::http::Header;

use crate::core::context::CtCrabContext;

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
fn http404catcher() -> api::APIError {
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

  fn on_response(&self, _request: &Request, response: &mut Response) {
    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  let ctx = CtCrabContext::new();
  {
    use crate::schema::ctlogs::dsl::*;
    let db = ctx.db()?;
    let count: usize = TryFrom::<i64>::try_from(ctlogs.select(count_star()).filter(monitoring.eq(true)).first(&db)
        .map_err(Box::new)?).unwrap();
    if count == 0 {
      #[derive(Debug, Error)]
      #[error("Failed to initialize ctlogs table: {0}")]
      struct E(#[source] core::initialise_ctlogs_table::E);
      core::initialise_ctlogs_table::initialise_or_update_ctlogs_table(&db)
          .map_err(|e| Box::new(E(e)))?;
      std::thread::sleep(Duration::from_millis(200)); // to give db time to sync changes
    }
  }
  ctx.init_update_threads();
  Err(Box::new(rocket::ignite()
      .mount("/", api::api_routes())
      .register(catchers![http500catcher, http404catcher])
      .attach(AccessControlFairing)
      .manage(ctx).launch()))
}
