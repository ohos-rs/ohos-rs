use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::*;

fn normalize_encoding(encoding: Option<String>) -> String {
  encoding
    .unwrap_or_else(|| "utf8".to_owned())
    .to_ascii_lowercase()
}

fn encode_string(input: String, encoding: Option<String>) -> Result<Vec<u8>> {
  match normalize_encoding(encoding).as_str() {
    "utf8" | "utf-8" => Ok(input.into_bytes()),
    "ascii" | "latin1" | "binary" => Ok(input.bytes().collect()),
    other => Err(Error::new(
      Status::InvalidArg,
      format!("Unsupported buffer encoding: {other}"),
    )),
  }
}

#[napi]
pub fn from_string<'env>(
  env: &'env Env,
  input: String,
  encoding: Option<String>,
) -> Result<ArrayBuffer<'env>> {
  ArrayBuffer::from_data(env, encode_string(input, encoding)?)
}

#[napi]
pub fn alloc<'env>(env: &'env Env, size: u32) -> Result<ArrayBuffer<'env>> {
  ArrayBuffer::from_data(env, vec![0; size as usize])
}

#[napi]
pub fn to_string(input: Uint8Array, encoding: Option<String>) -> Result<String> {
  match normalize_encoding(encoding).as_str() {
    "utf8" | "utf-8" => Ok(String::from_utf8_lossy(&input.to_vec()).into_owned()),
    "ascii" | "latin1" | "binary" => Ok(input.to_vec().into_iter().map(char::from).collect()),
    other => Err(Error::new(
      Status::InvalidArg,
      format!("Unsupported buffer encoding: {other}"),
    )),
  }
}
