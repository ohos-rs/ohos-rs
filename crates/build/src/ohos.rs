use std::{
  env,
  io::{Error, ErrorKind},
};

pub fn setup() -> Result<(), Error> {
  let target = env::var("TARGET").expect("Try to get build target failed.");
  let lib_dir = match target.as_ref() {
    "aarch64-unknown-linux-ohos" => "aarch64-linux-ohos",
    "armv7-unknown-linux-ohos" => "arm-linux-ohos",
    "x86_64-unknown-linux-ohos" => "x86_64-linux-ohos",
    _ => "",
  };

  if let Ok(hos_ndk) = env::var("HOS_NDK_HOME") {
    // for zig-build avoid to use RUSTFLAGS="-L xxxx"
    println!(
      "cargo:rustc-link-search={}/native/sysroot/usr/lib/{}",
      &hos_ndk, &lib_dir
    );

    // for libc++_shared.so etc.
    println!(
      "cargo:rustc-link-search={}/native/llvm/lib/{}",
      &hos_ndk, &lib_dir
    );
  } else if let Ok(ohos_ndk) = env::var("OHOS_NDK_HOME") {
    // for zig-build avoid to use RUSTFLAGS="-L xxxx"
    println!(
      "cargo:rustc-link-search={}/native/sysroot/usr/lib/{}",
      &ohos_ndk, &lib_dir
    );

    // for libc++_shared.so etc.
    println!(
      "cargo:rustc-link-search={}/native/llvm/lib/{}",
      &ohos_ndk, &lib_dir
    );
  } else {
    return Err(Error::new(
      ErrorKind::NotFound,
      "HOS_NDK_HOME or OHOS_NDK_HOME not set.",
    ));
  }

  // link libace_napi.z.so
  println!("cargo:rustc-link-lib=dylib=ace_napi.z");
  Ok(())
}
