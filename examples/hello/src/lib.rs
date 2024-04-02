use napi_derive_ohos::napi;
use napi_ohos::{bindgen_prelude::*, JsNumber, threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode, UnknownReturnValue}};

#[napi]
pub fn sum(left: i32, right: i32) -> i32 {
  left + right
}

#[napi]
pub fn threadsafe_function_fatal_mode(
  v: bool,
  cb: ThreadsafeFunction<bool, UnknownReturnValue, false>,
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

fn fib(n: u32) -> u32 {
  match n {
    0 => 0,
    1 => 1,
    _ => fib(n - 1) + fib(n - 2),
  }
}

struct AsyncFib {
  input: u32,
}

impl Task for AsyncFib {
  type Output = u32;
  type JsValue = JsNumber;

  fn compute(&mut self) -> Result<Self::Output> {
    Ok(fib(self.input))
  }

  fn resolve(&mut self, env: Env, output: u32) -> Result<Self::JsValue> {
    env.create_uint32(output)
  }
}

#[napi]
fn async_fib(input: u32, signal: AbortSignal) -> AsyncTask<AsyncFib> {
  AsyncTask::with_signal(AsyncFib { input }, signal)
}
