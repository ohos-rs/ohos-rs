fn main() {
  println!("cargo::rustc-check-cfg=cfg(tokio_unstable)");

  use napi_build_ohos::setup;

  setup();
}
