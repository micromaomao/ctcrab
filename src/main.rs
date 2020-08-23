#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;

use crate::core::context::CtCrabContext;

mod schema;
mod models;
mod core;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

fn main() {
  let ctx = match CtCrabContext::new() {
    Ok(ctx) => ctx,
    Err(e) => panic!("{}", e)
  };
  rocket::ignite().mount("/", routes![index]).manage(ctx).launch();
}
