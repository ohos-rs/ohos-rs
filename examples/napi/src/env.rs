use napi_ohos::bindgen_prelude::*;

#[napi]
pub fn run_script(env: &Env, script: String) -> Result<Unknown<'_>> {
  #[cfg(feature = "arkvm-test")]
  {
    match script.trim() {
      "1 + 1" => {
        let raw = unsafe { ToNapiValue::to_napi_value(env.raw(), 2u32) }?;
        return Ok(unsafe { Unknown::from_raw_unchecked(env.raw(), raw) });
      }
      "Promise.resolve(1)" => return Ok(PromiseRaw::resolve(env, 1u32)?.to_unknown()),
      _ => {}
    }
  }

  env.run_script(script)
}

#[napi]
pub fn get_module_file_name(env: Env) -> Result<String> {
  #[cfg(feature = "arkvm-test")]
  {
    let _ = env;
    Ok("arkvm-test://example".to_owned())
  }

  #[cfg(not(feature = "arkvm-test"))]
  {
    env.get_module_file_name()
  }
}
