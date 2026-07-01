use crate::util::Arch;
use flate2::read::GzDecoder;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::Archive;

const ARCHIVE_BYTES: &[u8] =
  include_bytes!("../../assets/atomic/OpenHarmony-6.0-Release-libatomic-ohos.tar.gz");
const ARCHIVE_NAME: &str = "OpenHarmony-6.0-Release-libatomic-ohos";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Linkage {
  Static,
  Dynamic,
}

impl Linkage {
  pub(crate) fn from_flags(sta_atomic: bool, dyn_atomic: bool) -> anyhow::Result<Option<Self>> {
    match (sta_atomic, dyn_atomic) {
      (false, false) => Ok(None),
      (true, false) => Ok(Some(Self::Static)),
      (false, true) => Ok(Some(Self::Dynamic)),
      (true, true) => Err(anyhow::Error::msg(
        "--sta-atomic and --dyn-atomic cannot be used together.",
      )),
    }
  }

  fn file_name(self) -> &'static str {
    match self {
      Self::Static => "libatomic.a",
      Self::Dynamic => "libatomic.so",
    }
  }

  fn dir_name(self) -> &'static str {
    match self {
      Self::Static => "static",
      Self::Dynamic => "dynamic",
    }
  }
}

pub(crate) struct Resolved {
  pub(crate) search_dir: PathBuf,
  pub(crate) lib: PathBuf,
}

pub(crate) fn resolve(
  cargo_target_dir: &Path,
  release: bool,
  arch: &Arch,
  linkage: Linkage,
) -> anyhow::Result<Resolved> {
  let lib = lib_path(cargo_target_dir, release, arch, linkage)?;
  let search_dir = prepare_search_dir(cargo_target_dir, release, arch, linkage, &lib)?;

  Ok(Resolved { search_dir, lib })
}

fn lib_path(
  cargo_target_dir: &Path,
  release: bool,
  arch: &Arch,
  linkage: Linkage,
) -> anyhow::Result<PathBuf> {
  let root = ensure_unpacked(cargo_target_dir)?;
  let profile = if release { "release" } else { "debug" };
  let lib = root
    .join(profile)
    .join(archive_abi(arch)?)
    .join(linkage.file_name());

  if !lib.is_file() {
    return Err(anyhow::Error::msg(format!(
      "Bundled OHOS {} not found for {} profile and {}: {}",
      linkage.file_name(),
      profile,
      arch.to_arch(),
      lib.display()
    )));
  }

  Ok(lib)
}

fn prepare_search_dir(
  cargo_target_dir: &Path,
  release: bool,
  arch: &Arch,
  linkage: Linkage,
  lib: &Path,
) -> anyhow::Result<PathBuf> {
  let profile = if release { "release" } else { "debug" };
  let search_dir = cargo_target_dir
    .join("ohrs")
    .join("atomic-link")
    .join(linkage.dir_name())
    .join(profile)
    .join(archive_abi(arch)?);

  fs::create_dir_all(&search_dir)?;
  fs::copy(lib, search_dir.join(linkage.file_name()))?;

  Ok(search_dir)
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
      "--sta-atomic and --dyn-atomic do not support loongarch64 because the bundled OHOS libatomic archive does not contain that ABI.",
    )),
  }
}
