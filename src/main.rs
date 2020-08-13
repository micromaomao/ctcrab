#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod schema;
mod models;
mod core;
use crate::core::open_db;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

fn main() {
  let db = open_db();
  rocket::ignite().mount("/", routes![index]).launch();
}
