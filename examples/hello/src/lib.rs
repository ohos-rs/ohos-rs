use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::pre_init;
use napi_ohos::module_init;

#[napi]
pub fn add(left: u32, right: Option<u32>) -> u32 {
  let r = right.unwrap_or(10);
  left + r
}


#[module_init]
fn init() {
  pre_init();
}
