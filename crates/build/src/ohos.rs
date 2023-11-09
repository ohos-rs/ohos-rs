use std::env;

pub fn setup() {
  let _ndk = env::var("OHOS_NDK_HOME").expect("OHOS_NDK_HOME not set");
  // link libace_napi.z.so
  println!("cargo:rustc-link-lib=dylib=ace_napi.z");
}
