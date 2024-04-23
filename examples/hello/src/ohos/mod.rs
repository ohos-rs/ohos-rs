use napi_derive_ohos::napi;
use napi_ohos::{ark::ArkRuntime, Env, Result};

#[napi]
pub fn load_module_with_default_env(env: Env) -> Result<()> {
    let runtime = ArkRuntime::new_with_env(env);
    let module = runtime.load("@ohos.log")?;
    let js_string = env.create_string("Hello Harmony".as_ref())?;
    module.call("info", &[js_string])?;
    Ok(())
}

#[napi]
pub fn load_module_with_new_runtime(env: Env) -> Result<()> {
    let runtime = ArkRuntime::new()?;
    let module = runtime.load("@ohos.log")?;
    // for new runtime, you should use runtime's env to call some method
    let js_string = runtime.env.create_string("Hello Harmony".as_ref())?;
    module.call("info", &[js_string])?;
    Ok(())
}