pub const CARGO_CONFIG_TOML: &str = r#"
[target.aarch64-unknown-linux-ohos]
ar = "$OH_SDK_HOME/native/llvm/bin/llvm-ar"
linker = "scripts/aarch64-unknown-linux-ohos-clang.sh"

[target.armv7-unknown-linux-ohos]
ar = "$OH_SDK_HOME/native/llvm/bin/llvm-ar"
linker = "scripts/armv7-unknown-linux-ohos-clang.sh"

[target.x86_64-unknown-linux-ohos]
ar = "$OH_SDK_HOME/native/llvm/bin/llvm-ar"
linker = "scripts/x86_64-unknown-linux-ohos-clang.sh"
"#;

pub const ARM64_C_BUILD_SHELL: &str = r#"
#!/bin/sh
exec $OHOS_NDK_HOME/native/llvm/bin/clang \
  -target aarch64-linux-ohos \
  --sysroot=$OHOS_NDK_HOME/native/sysroot \
  -D__MUSL__ \
  "$@"

"#;
pub const ARM64_CPP_BUILD_SHELL: &str = r#"
#!/bin/sh
exec $OHOS_NDK_HOME/native/llvm/bin/clang++ \
  -target aarch64-linux-ohos \
  --sysroot=$OHOS_NDK_HOME/native/sysroot \
  -D__MUSL__ \
  "$@"

"#;
pub const ARM_C_BUILD_SHELL: &str = r#"
#!/bin/sh
exec $OHOS_NDK_HOME/native/llvm/bin/clang \
  -target arm-linux-ohos \
  --sysroot=$OHOS_NDK_HOME/native/sysroot \
  -D__MUSL__ \
  -march=armv7-a \
  -mfloat-abi=softfp \
  -mtune=generic-armv7-a \
  -mthumb \
  "$@"

"#;
pub const ARM_CPP_BUILD_SHELL: &str = r#"
#!/bin/sh
exec $OHOS_NDK_HOME/native/llvm/bin/clang++ \
  -target arm-linux-ohos \
  --sysroot=$OHOS_NDK_HOME/native/sysroot \
  -D__MUSL__ \
  -march=armv7-a \
  -mfloat-abi=softfp \
  -mtune=generic-armv7-a \
  -mthumb \
  "$@"

"#;
pub const X86_64_C_BUILD_SHELL: &str = r#"
#!/bin/sh
exec $OHOS_NDK_HOME/native/llvm/bin/clang \
  -target x86_64-linux-ohos \
  --sysroot=$OHOS_NDK_HOME/native/sysroot \
  -D__MUSL__ \
  "$@"

"#;
pub const X86_64_CPP_BUILD_SHELL: &str = r#"
#!/bin/sh
exec $OHOS_NDK_HOME/native/llvm/bin/clang++ \
  -target x86_64-linux-ohos \
  --sysroot=$OHOS_NDK_HOME/native/sysroot \
  -D__MUSL__ \
  "$@"
"#;
pub const BUILD_INIT: &str = r#"
extern crate napi_build_ohos;

fn main() {
  napi_build_ohos::setup();
}

"#;
pub const CARGO_TOML: &str = r#"
[package]
name    = "entry"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[lib]
crate-type = ["cdylib"]

[dependencies]
napi-ohos        = { path = '../../crates/napi' }
napi-derive-ohos = { path = '../../crates/macro' }

[build-dependencies]
napi-build-ohos = { path = '../../crates/build' }

"#;

pub const LIB_CODE: &str = r#"
use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::pre_init;
use napi_ohos::module_init;

#[napi]
pub fn add(left: u32, right: u32) -> u32 {
  left + right
}
#[module_init]
fn init() {
  pre_init();
}
"#;
