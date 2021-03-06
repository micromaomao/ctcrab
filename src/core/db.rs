use diesel::{Connection, PgConnection};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::Error;

fn get_url_from_env() -> String {
  dotenv::dotenv().ok();
  std::env::var("DATABASE_URL").map_err(|_| "Expected DATABASE_URL env.").unwrap()
}

pub type DBConn = PgConnection;
pub fn open_db() -> DBConn {
  DBConn::establish(&get_url_from_env()).unwrap()
}

pub type DBPool = Pool<ConnectionManager<DBConn>>;
pub type DBPooledConn = PooledConnection<ConnectionManager<DBConn>>;
pub fn create_db_pool() -> DBPool {
  Pool::builder().max_size(20).build(ConnectionManager::new(&get_url_from_env())).unwrap()
}

use diesel::result::Error as DieselError;
use std::time::Duration;

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
        Err(_) if nb_tries < 5 => {
          nb_tries += 1;
          std::thread::sleep(Duration::from_millis(5u64 * 2u64.pow(nb_tries)));
          continue;
        },
        Err(RunErr::Db(db_err @ diesel::result::Error::DatabaseError(_, _))) => panic!("db error: {}. May cause connection corruption, hence panicking.", db_err),
        Err(RunErr::Db(e)) => return Err(e.into()),
        Err(RunErr::User(user_err)) => return Err(user_err)
      }
    }
  }
}
