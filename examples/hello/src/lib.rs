use napi_derive_ohos::napi;
use napi_ohos::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode, UnknownReturnValue},
};

mod async_work;
mod ohos;

#[napi]
pub fn sum(left: i32, right: i32) -> i32 {
  left + right
}

#[napi]
pub fn threadsafe_function_fatal_mode(
  v: bool,
  cb: ThreadsafeFunction<bool, UnknownReturnValue, bool, false>,
) -> Result<()> {
  match v {
    true => {
      cb.call(true, ThreadsafeFunctionCallMode::NonBlocking);
      Ok(())
    }
    false => Err(Error::new(
      Status::GenericFailure,
      "ThrowFromNative".to_owned(),
    )),
  }
}

#[napi]
pub struct Utils;

#[napi]
impl Utils {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }

  #[napi]
  pub fn sum(&self, left: i32, right: i32) -> i32 {
    left + right
  }
}
