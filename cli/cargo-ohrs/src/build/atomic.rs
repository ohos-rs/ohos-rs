use crate::util::Arch;
use flate2::read::GzDecoder;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::Archive;

const ARCHIVE_BYTES: &[u8] =
  include_bytes!("../../assets/atomic/OpenHarmony-6.0-Release-libatomic-ohos.tar.gz");
const ARCHIVE_NAME: &str = "OpenHarmony-6.0-Release-libatomic-ohos";

pub(crate) fn lib_path(
  cargo_target_dir: &Path,
  release: bool,
  arch: &Arch,
) -> anyhow::Result<PathBuf> {
  let root = ensure_unpacked(cargo_target_dir)?;
  let profile = if release { "release" } else { "debug" };
  let lib = root
    .join(profile)
    .join(archive_abi(arch)?)
    .join("libatomic.a");

  if !lib.is_file() {
    return Err(anyhow::Error::msg(format!(
      "Bundled OHOS libatomic not found for {} profile and {}: {}",
      profile,
      arch.to_arch(),
      lib.display()
    )));
  }

  Ok(lib)
}

fn ensure_unpacked(cargo_target_dir: &Path) -> anyhow::Result<PathBuf> {
  let root = cargo_target_dir
    .join("ohrs")
    .join("atomic")
    .join(ARCHIVE_NAME);
  let marker = root.join(".unpacked");
  if marker.is_file() {
    return Ok(root);
  }

  fs::create_dir_all(&root)?;
  let decoder = GzDecoder::new(Cursor::new(ARCHIVE_BYTES));
  let mut archive = Archive::new(decoder);
  archive.unpack(&root)?;
  fs::write(marker, ARCHIVE_NAME)?;

  Ok(root)
}

fn archive_abi(arch: &Arch) -> anyhow::Result<&'static str> {
  match arch {
    Arch::ARM64 => Ok("arm64-v8a"),
    Arch::ARM32 => Ok("armeabi-v7a"),
    Arch::X86_64 => Ok("x86_64"),
    Arch::LoongArch64 => Err(anyhow::Error::msg(
      "--atomic does not support loongarch64 because the bundled OHOS libatomic archive does not contain that ABI.",
    )),
  }
}
