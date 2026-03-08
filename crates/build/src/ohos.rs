use std::env;
use std::path::Path;

pub fn setup() {
  let ndk = env::var("OHOS_NDK_HOME").expect("OHOS_NDK_HOME not set.");
  let target = env::var("TARGET").expect("Try to get build target failed.");
  let lib_dir = match target.as_ref() {
    "aarch64-unknown-linux-ohos" => "aarch64-linux-ohos",
    "armv7-unknown-linux-ohos" => "arm-linux-ohos",
    "x86_64-unknown-linux-ohos" => "x86_64-linux-ohos",
    _ => "",
  };
  // for zig-build avoid to use RUSTFLAGS="-L xxxx"
  println!(
    "cargo:rustc-link-search={}/native/sysroot/usr/lib/{}",
    &ndk, &lib_dir
  );
  // for libc++_shared.so etc.
  println!(
    "cargo:rustc-link-search={}/native/llvm/lib/{}",
    &ndk, &lib_dir
  );
  // link libace_napi.z.so
  println!("cargo:rustc-link-lib=dylib=ace_napi.z");
}

pub fn setup_arkvm_test() {
  println!("cargo:rerun-if-env-changed=ARK_HOST_BUNDLE_DIR");
  println!("cargo:rerun-if-env-changed=ARK_ACE_NAPI_LIB");

  let ace_napi_lib = env::var("ARK_ACE_NAPI_LIB")
    .ok()
    .map(Into::into)
    .or_else(|| env::var("ARK_HOST_BUNDLE_DIR").ok().map(|dir| Path::new(&dir).join("libace_napi.so")))
    .expect("ARK_HOST_BUNDLE_DIR or ARK_ACE_NAPI_LIB must be set when feature `arkvm-test` is enabled");
  let link_dir = ace_napi_lib
    .parent()
    .expect("ARK_ACE_NAPI_LIB must point to a file inside a directory");

  let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

  println!("cargo:rustc-link-search=native={}", link_dir.display());
  if target_os == "linux" {
    println!("cargo:rustc-cdylib-link-arg=-Wl,--no-as-needed");
    println!("cargo:rustc-cdylib-link-arg={}", ace_napi_lib.display());
    println!("cargo:rustc-cdylib-link-arg=-Wl,--as-needed");
  } else {
    println!("cargo:rustc-link-lib=dylib=ace_napi");
  }
}
