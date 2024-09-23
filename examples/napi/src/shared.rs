use napi_derive_ohos::napi;
use shared::Shared;

#[napi]
pub fn return_from_shared_crate() -> Shared {
  Shared { value: 42 }
}
