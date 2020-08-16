use diesel::{Connection, PgConnection, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::Error;

pub type DBConn = PgConnection;
pub fn open_db() -> DBConn {
  dotenv::dotenv().ok();
  let db_url = std::env::var("DATABASE_URL").unwrap();
  DBConn::establish(&db_url).unwrap()
}

pub fn create_db_pool() -> Pool<ConnectionManager<DBConn>> {
  let db_url = std::env::var("DATABASE_URL").unwrap();
  Pool::builder().max_size(1).build(ConnectionManager::new(&db_url)).unwrap()
}

use diesel::result::Error as DieselError;

pub trait PgConnectionHelper {
  fn transaction_rw_serializable<T, E: From<DieselError>, F: FnMut() -> Result<T, E>>(&self, f: F) -> Result<T, E>;
}

impl PgConnectionHelper for PgConnection {
  fn transaction_rw_serializable<T, E: From<DieselError>, F: FnMut() -> Result<T, E>>(&self, mut f: F) -> Result<T, E> {
    enum RunErr<E> {
      User(E),
      Db(diesel::result::Error)
    };
    impl<E> From<diesel::result::Error> for RunErr<E> {
      fn from(e: Error) -> Self {
        RunErr::Db(e)
      }
    }
    let mut nb_tries = 0;
    loop {
      let res = self.build_transaction().read_write().serializable().run(|| {
        f().map_err(RunErr::User)
      });
      match res {
        Ok(r) => return Ok(r),
        Err(RunErr::Db(diesel_err @ diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::SerializationFailure, _))) => {
          nb_tries += 1;
          if nb_tries > 5 {
            return Err(diesel_err.into());
          } else {
            std::thread::yield_now();
            continue;
          }
        },
        Err(RunErr::Db(db_err @ diesel::result::Error::DatabaseError(_, _))) => panic!("db error: {}. May cause connection corruption, hence panicking.", db_err),
        Err(RunErr::Db(e)) => return Err(e.into()),
        Err(RunErr::User(user_err)) => return Err(user_err)
      }
    }
  }
}
