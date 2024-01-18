use std::env;

pub fn setup() {
  let _ndk = env::var("OHOS_NDK_HOME").expect("OHOS_NDK_HOME not set");
  let name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME should not be empty");
  println!("cargo:rustc-env=NAPI_BUILD_TARGET_NAME={}", name.as_str());
  // link libace_napi.z.so
  println!("cargo:rustc-link-lib=dylib=ace_napi.z");
}
