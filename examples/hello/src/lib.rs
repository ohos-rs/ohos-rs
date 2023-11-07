use napi::bindgen_prelude::pre_init;
use napi::module_init;
use napi_derive::napi;

#[napi]
pub fn add(left: u32, right: u32) -> u32 {
  left + right
}

#[napi]
pub fn subtraction(left: u32, right: u32) -> u32 {
  left - right
}

#[module_init]
fn init() {
  pre_init()
}
