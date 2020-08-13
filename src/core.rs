pub fn open_db() -> diesel::SqliteConnection {
  use diesel::Connection;
  dotenv::dotenv().ok();
  let db_url = std::env::var("DATABASE_URL").unwrap();
  diesel::sqlite::SqliteConnection::establish(&db_url).unwrap()
}
