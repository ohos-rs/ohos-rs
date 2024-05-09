use napi_derive_ohos::napi;

#[napi(object)]
pub struct Shared {
  pub value: u32,
}
