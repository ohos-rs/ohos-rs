use napi_derive_ohos::napi;
use napi_ohos::{ark::ArkRuntime, Env, JsString, Result};

#[napi]
pub fn load_module_with_default_env(env: Env) -> Result<()> {
  let runtime = ArkRuntime::new_with_env(env);
  let module = runtime.load("@ohos.log")?;
  let js_string = runtime
    .env
    .create_string("Hello Harmony".as_ref())?
    .into_unknown();
  let tag = runtime.env.create_string("test".as_ref())?.into_unknown();
  let flag = runtime.env.create_int32(0)?.into_unknown();
  module.call("info", &[flag, tag, js_string])?;
  Ok(())
}

#[napi]
pub fn load_module_with_new_runtime() -> Result<()> {
  let runtime = ArkRuntime::new()?;
  let module = runtime.load("@ohos.log")?;
  // for new runtime, you should use runtime's env to call some method
  let js_string = runtime
    .env
    .create_string("Hello Harmony".as_ref())?
    .into_unknown();
  let tag = runtime.env.create_string("test".as_ref())?.into_unknown();
  let flag = runtime.env.create_int32(0)?.into_unknown();
  module.call("info", &[flag, tag, js_string])?;
  Ok(())
}

#[napi]
pub fn load_module_with_field(env: Env) -> Result<JsString> {
  let runtime = ArkRuntime::new_with_env(env);
  let module = runtime.load("ets/Test")?;
  let word: JsString = module.get("hello")?;
  Ok(word)
}
