use napi_derive_ohos::napi;
use napi_shared::Shared;

#[napi]
pub fn return_from_shared_crate() -> Shared {
  Shared { value: 42 }
}
