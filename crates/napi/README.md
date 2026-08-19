# ohos-rs

![Crates.io Version](https://img.shields.io/crates/v/napi-ohos) ![Platform](https://img.shields.io/badge/platform-arm64/arm/x86__64-blue) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> This project is forked from [napi-rs](https://github.com/napi-rs/napi-rs), many thank to [@Brooooooklyn](https://github.com/Brooooooklyn).

## MSRV

1.88.0

## Taste

> You can use [ohrs](https://ohos.rs/en/docs/cli/init.html) to start a new project.

### Define ArkTS function

```rs
use napi_ohos::bindgen_prelude::*;
use napi_derive_ohos::napi;

/// module registration is done by the runtime, no need to explicitly do it now.
#[napi]
pub fn fibonacci(n: u32) -> u32 {
  match n {
    1 | 2 => 1,
    _ => fibonacci(n - 1) + fibonacci(n - 2),
  }
}

/// use `Fn`, `FnMut` or `FnOnce` traits to defined JavaScript callbacks
/// the return type of callbacks can only be `Result`.
#[napi]
pub fn get_cwd<T: Fn(String) -> Result<()>>(callback: T) {
  callback(
    std::env::current_dir()
      .unwrap()
      .to_string_lossy()
      .to_string(),
  )
  .unwrap();
}

/// or, define the callback signature in where clause
#[napi]
pub fn test_callback<T>(callback: T) -> Result<()>
where
  T: Fn(String) -> Result<()>,
{
  callback(std::env::current_dir()?.to_string_lossy().to_string())
}

/// async fn, require `async` feature enabled.
/// [dependencies]
/// napi = {version="2", features=["async"]}
#[napi]
pub async fn read_file_async(path: String) -> Result<Buffer> {
  Ok(tokio::fs::read(path).await?.into())
}
```

### Task / AsyncTask

Offload CPU work to the libuv thread pool and return a `Promise` to ArkTS. Implement [`Task`](https://docs.rs/napi-ohos/latest/napi_ohos/trait.Task.html), then return [`AsyncTask`](https://docs.rs/napi-ohos/latest/napi_ohos/bindgen_prelude/struct.AsyncTask.html). `JsObject` is not exported from `bindgen_prelude`. `Env::spawn(task)?.promise_object()` returns [`PromiseRaw`](https://docs.rs/napi-ohos/latest/napi_ohos/bindgen_prelude/struct.PromiseRaw.html), not `Object`.

```rs
use napi_ohos::bindgen_prelude::*;
use napi_derive_ohos::napi;

fn fibonacci_native(n: u32) -> u32 {
  match n {
    1 | 2 => 1,
    _ => fibonacci_native(n - 1) + fibonacci_native(n - 2),
  }
}

struct ComputeFib {
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

#[napi]
pub fn fib(init: u32) -> AsyncTask<ComputeFib> {
  AsyncTask::new(ComputeFib::new(init))
}

#[napi]
pub fn spawn_fib<'env>(env: &'env Env, init: u32) -> Result<PromiseRaw<'env, u32>> {
  Ok(env.spawn(ComputeFib::new(init))?.promise_object())
}
```

See `examples/napi/src/task.rs` and `examples/hello/src/async_work/mod.rs`.

## Building

Before build, we must setup some environments. You can follow the [document](https://ohos.rs/en/docs/basic/quick-start.html) to setup them.

Then you can use `ohrs` to build it directly.

```sh
ohrs build

# build single arch
ohrs build --arch aarch
```

Finally you can copy the `dist` folder into your OpenHarmony/HarmonyNext project and use it.

## Asynchronous runtime

We use `tokio` as the default asynchronous runtime. But for some simple scenarios, we don't need so complete runtime, and you can try [ohos-ffrt](https://github.com/ohos-rs/ohos-ffrt).

## Discussion

[Feel free to join our WeChat group!](https://github.com/ohos-rs/ohos-rs/wiki/Welcome-to-join-our-WeChat-Group!)

## License

[MIT](https://github.com/ohos-rs/ohos-rs/blob/ohos/LICENSE)
