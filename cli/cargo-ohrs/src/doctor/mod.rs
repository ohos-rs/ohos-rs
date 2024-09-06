use owo_colors::OwoColorize;
use std::env;

use render_result::render;
use semver::Version;
use target::{resolve_rust_version, resolve_targets};

use crate::util::Arch;
mod render_result;
mod target;

pub fn doctor() -> anyhow::Result<()> {
  let targets = resolve_targets()?;
  let origin_version = resolve_rust_version()?;
  let version = Version::parse(&origin_version)?;
  let ndk = env::var("OHOS_NDK_HOME").unwrap_or_default();

  let msvc = Version::parse("1.78.0")?;

  let is_env_ok = !ndk.is_empty();
  println!(
    "{}  Environment variable {} should be set.",
    render(is_env_ok),
    "OHOS_NDK_HOME".green()
  );
  println!(
    "{}  Rust version should be >= 1.78.0.",
    render(version >= msvc)
  );
  [Arch::ARM64, Arch::ARM32, Arch::X86_64]
    .iter()
    .for_each(|arch| {
      let t = arch.rust_target();
      println!(
        "{}  Rustup target: {} should be installed.",
        render((&targets).contains(&t.to_owned())),
        t.green(),
      );
    });
  Ok(())
}
