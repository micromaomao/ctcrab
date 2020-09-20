use std::error::Error;
use std::mem::MaybeUninit;
use std::sync::{Mutex, Arc};

use crate::core::db::{create_db_pool, DBPool, DBPooledConn};
use crate::core::update_thread;

pub struct CtCrabContext {
  db_pool: DBPool,
  update_threads: Mutex<Vec<update_thread::Handle>>
}

impl CtCrabContext {
  pub fn new() -> Result<CtCrabContext, Box<dyn Error>> {
    let ctx = CtCrabContext {
      db_pool: create_db_pool(),
      update_threads: Mutex::new(Vec::new())
    };
    ctx.init_update_threads()?;
    Ok(ctx)
  }

  pub fn db(&self) -> Result<DBPooledConn, Box<dyn Error>> {
    self.db_pool.get().map_err(|x| Box::new(x) as _)
  }

  fn init_update_threads(&self) -> Result<(), Box<dyn Error>> {
    let mut update_threads = self.update_threads.lock().unwrap();
    update_threads.truncate(0);
    use diesel::prelude::*;
    use crate::schema::ctlogs::dsl::*;
    use crate::models::CtLog;
    let logs: Vec<CtLog> = ctlogs.filter(monitoring.eq(true)).load(&self.db()?)?;
    for l in logs {
      // Safety: we pass ing a &'static DBPool so that threading works. However
      // all threads using the DBPool will exit before self, and hence the DBPool, is actually
      // dropped.
      let hdl = update_thread::init_thread(self.db_pool.clone(), l);
      update_threads.push(hdl);
    }
    Ok(())
  }
}

impl Drop for CtCrabContext {
  fn drop(&mut self) {
    // Drop impl of Handle wait for the threads to terminate.
    self.update_threads.lock().unwrap().truncate(0);
  }
}
