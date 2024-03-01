use napi_derive_ohos::napi;

#[napi]
pub fn add(left: u32, right: Option<u32>) -> u32 {
  let r = right.unwrap_or(10);
  left + r
}

#[napi]
pub fn get_string() -> String {
  String::from("hello world")
}

#[napi]
pub struct Test(String);

#[napi]
impl Test {
  #[napi(constructor)]
  pub fn new(s: String) -> Self {
    Test(s)
  }
}