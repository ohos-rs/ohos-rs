use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::*;

fn fibonacci_native(n: u32) -> u32 {
  match n {
    0 => 0,
    1 => 1,
    _ => fibonacci_native(n - 1) + fibonacci_native(n - 2),
  }
}

pub struct ComputeFib {
  n: u32,
}

impl ComputeFib {
  pub fn new(n: u32) -> Self {
    Self { n }
  }
}

impl Task for ComputeFib {
  type Output = u32;
  type JsValue = u32;

  fn compute(&mut self) -> Result<Self::Output> {
    Ok(fibonacci_native(self.n))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

/// Preferred bindgen pattern: return `AsyncTask<T>`, which becomes `Promise<number>`.
#[napi]
pub fn fib(init: u32) -> AsyncTask<ComputeFib> {
  AsyncTask::new(ComputeFib::new(init))
}

/// Manual `Env::spawn` returns `AsyncWorkPromise`; `promise_object()` is `PromiseRaw`.
#[napi]
pub fn spawn_fib<'env>(env: &'env Env, init: u32) -> Result<PromiseRaw<'env, u32>> {
  Ok(env.spawn(ComputeFib::new(init))?.promise_object())
}

pub struct AsyncFib {
  input: u32,
}

impl Task for AsyncFib {
  type Output = u32;
  type JsValue = u32;

  fn compute(&mut self) -> Result<Self::Output> {
    Ok(fibonacci_native(self.input))
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
