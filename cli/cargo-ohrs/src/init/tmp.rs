pub const BUILD_INIT: &str = r#"fn main() {
  napi_build_ohos::setup();
}
"#;
pub const CARGO_TOML: &str = r#"[package]
name    = "entry"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[lib]
crate-type = ["cdylib"]

[dependencies]
napi-ohos        = { version = "*" }
napi-derive-ohos = { version = "*" }

[build-dependencies]
napi-build-ohos = { version = "*" }

[profile.release]
lto = true
"#;

pub const LIB_CODE: &str = r#"use napi_derive_ohos::napi;

#[napi]
pub fn add(left: u32, right: u32) -> u32 {
  left + right
}
"#;

pub const GIT_IGNORE: &str = r#"dist/
target/
.DS_Store
.idea/

*.har

Cargo.lock
"#;
