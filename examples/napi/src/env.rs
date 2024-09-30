use napi_ohos::bindgen_prelude::*;

#[napi]
pub fn run_script(env: Env, script: String) -> Result<Unknown> {
  env.run_script(script)
}

#[napi]
pub fn get_module_file_name(env: Env) -> Result<String> {
  env.get_module_file_name()
}
