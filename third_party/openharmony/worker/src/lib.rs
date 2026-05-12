use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::*;
use serde_json::Value;

#[derive(Clone)]
struct WorkerRecord {
  script_url: String,
  name: Option<String>,
}

static NEXT_WORKER_ID: AtomicU32 = AtomicU32::new(1);
static WORKERS: OnceLock<Mutex<HashMap<u32, WorkerRecord>>> = OnceLock::new();

fn workers() -> &'static Mutex<HashMap<u32, WorkerRecord>> {
  WORKERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn worker_error(message: impl Into<String>) -> Error {
  Error::new(Status::GenericFailure, message.into())
}

#[napi]
pub fn create_thread_worker(script_url: String, name: Option<String>) -> u32 {
  let id = NEXT_WORKER_ID.fetch_add(1, Ordering::Relaxed);
  workers()
    .lock()
    .unwrap()
    .insert(id, WorkerRecord { script_url, name });
  id
}

#[napi]
pub fn terminate_thread_worker(id: u32) -> bool {
  workers().lock().unwrap().remove(&id).is_some()
}

#[napi]
pub fn get_thread_worker_script_url(id: u32) -> Result<String> {
  workers()
    .lock()
    .unwrap()
    .get(&id)
    .map(|record| record.script_url.clone())
    .ok_or_else(|| worker_error(format!("Worker {id} is not available")))
}

#[napi]
pub fn get_thread_worker_name(id: u32) -> Result<Option<String>> {
  workers()
    .lock()
    .unwrap()
    .get(&id)
    .map(|record| record.name.clone())
    .ok_or_else(|| worker_error(format!("Worker {id} is not available")))
}

#[napi]
pub async fn post_message(id: u32, _message: Value) -> Result<()> {
  let exists = workers().lock().unwrap().contains_key(&id);
  if !exists {
    return Err(worker_error(format!("Worker {id} is not available")));
  }

  tokio::task::yield_now().await;
  Ok(())
}
