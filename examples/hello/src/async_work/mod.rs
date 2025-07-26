use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::*;

fn fib(n: u32) -> u32 {
  match n {
    0 => 0,
    1 => 1,
    _ => fib(n - 1) + fib(n - 2),
  }
}

pub struct AsyncFib {
  input: u32,
}

impl Task for AsyncFib {
  type Output = u32;
  type JsValue = u32;

  fn compute(&mut self) -> Result<Self::Output> {
    Ok(fib(self.input))
  }

  fn resolve(&mut self, _env: napi_ohos::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

#[napi]
pub fn async_fib(input: u32, signal: AbortSignal) -> AsyncTask<AsyncFib> {
  AsyncTask::with_signal(AsyncFib { input }, signal)
}

#[napi]
pub fn async_fib_qos(input: u32) -> AsyncTask<AsyncFib> {
  AsyncTask::with_qos(AsyncFib { input }, napi_ohos::AsyncWorkQos::Utility)
}
