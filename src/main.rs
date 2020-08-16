#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod schema;
mod models;
mod core;
use crate::core::create_db_pool;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

fn main() {
  let db_pool = create_db_pool();
  rocket::ignite().mount("/", routes![index]).manage(db_pool).launch();
}
