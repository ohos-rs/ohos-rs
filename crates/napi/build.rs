use std::env;

fn main() {
  if env::var("CARGO_FEATURE_NAPI9").is_ok() {
    panic!("Please don't set features with napi9")
  }

  if env::var("CARGO_FEATURE_EXPERIMENTAL").is_ok() {
    panic!("Please don't set features with experimental")
  }
  println!("cargo::rustc-check-cfg=cfg(tokio_unstable)");

  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
  if target_os == "windows" && target_env == "gnu" {
    napi_build_ohos::setup();
  }
}
