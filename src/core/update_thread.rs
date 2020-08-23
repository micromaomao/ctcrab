use std::error::Error;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use log::info;

use crate::core::db::{DBPool, DBPooledConn, PgConnectionHelper};
use crate::models::{Hash, CtLog};
use std::mem::{MaybeUninit, replace};
use diesel::Connection;
use std::panic::AssertUnwindSafe;

enum ChannelMessage {
  Stop
}

pub struct Handle {
  jh: MaybeUninit<JoinHandle<()>>,
  sender: mpsc::Sender<ChannelMessage>
}

impl Drop for Handle {
  fn drop(&mut self) {
    self.sender.send(ChannelMessage::Stop).unwrap();
    unsafe { replace(&mut self.jh, MaybeUninit::uninit()).assume_init() }.join().unwrap();
  }
}

pub fn init_thread(db_pool: &'static DBPool, log: CtLog) -> Handle {
  let (sender, recv) = mpsc::channel::<ChannelMessage>();
  let jh = thread::Builder::new().name(format!("update-{}", &log.log_id)).spawn(move || {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
      loop {
        let db_conn = match db_pool.get() {
          Ok(k) => k,
          Err(e) => panic!("Error opening db: {}", e)
        };
        info!("Updating log {} ({})", &log.log_id, &log.name);
        match db_conn.transaction_rw_serializable::<(), diesel::result::Error, _>(|| {
          Ok(())
        }) {
          Ok(()) => {},
          Err(e) => {
            panic!("Database error while updating: {}", e);
          }
        }
        drop(db_conn);
        match recv.recv_timeout(Duration::from_secs(5)) {
          Err(mpsc::RecvTimeoutError::Timeout) => {},
          r @ Err(_) => { r.unwrap(); },
          Ok(msg) => {
            match msg {
              ChannelMessage::Stop => {
                info!("Thread for log {} exiting", &log.log_id);
                return;
              }
            }
          }
        }
      }
    })) {
      Ok(()) => {},
      Err(_) => {
        std::process::abort();
      }
    }
  }).unwrap();
  Handle { jh: MaybeUninit::new(jh), sender }
}

