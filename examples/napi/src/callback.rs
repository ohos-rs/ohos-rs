use std::{env, format};

use napi_ohos::bindgen_prelude::*;

#[napi]
fn get_cwd<T: Fn(String) -> Result<()>>(callback: T) {
  callback(env::current_dir().unwrap().to_string_lossy().to_string()).unwrap();
}

#[napi]
fn option_end<T: Fn(String, Option<String>) -> Result<()>>(callback: T) {
  callback("Hello".to_string(), None).unwrap();
}

#[napi]
fn option_start<T: Fn(Option<String>, String) -> Result<()>>(callback: T) {
  callback(None, "World".to_string()).unwrap();
}

#[napi]
fn option_start_end<T: Fn(Option<String>, String, Option<String>) -> Result<()>>(callback: T) {
  callback(None, "World".to_string(), None).unwrap();
}

#[napi]
fn option_only<T: Fn(Option<String>) -> Result<()>>(callback: T) {
  callback(None).unwrap();
}

/// napi = { version = 2, features = ["serde-json"] }
#[napi]
fn read_file<T: Fn(Result<()>, Option<String>) -> Result<()>>(callback: T) {
  match read_file_content() {
    Ok(s) => callback(Ok(()), Some(s)),
    Err(e) => callback(Err(e), None),
  }
  .unwrap();
}

fn read_file_content() -> Result<String> {
  // serde_json::from_str(&s)?;
  Ok("hello world".to_string())
}

#[napi(ts_return_type = "Promise<string>")]
pub fn callback_return_promise_and_spawn<F: Fn(String) -> Result<Promise<String>>>(
  env: &Env,
  js_func: F,
) -> napi_ohos::Result<PromiseRaw<String>> {
  let promise = js_func("Hello".to_owned())?;
  env.spawn_future(async move {
    let resolved = promise.await?;
    Ok::<String, napi_ohos::Error>(format!("{} 😼", resolved))
  })
}

#[napi]
pub fn capture_error_in_callback<C: Fn() -> Result<()>, E: Fn(Error) -> Result<()>>(
  cb1: C,
  cb2: E,
) -> Result<()> {
  if let Err(e) = cb1() {
    cb2(e)
  } else {
    Ok(())
  }
}
