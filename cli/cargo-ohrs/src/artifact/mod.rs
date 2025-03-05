use crate::artifact::tgz::generate_har;
use crate::check_and_clean_file_or_dir;
use anyhow::Error;
use fs_extra::dir::CopyOptions;
use std::fs;
use std::{env, path::PathBuf};

mod tgz;

pub fn artifact(args: crate::ArtifactArgs) -> anyhow::Result<()> {
  let pwd = env::current_dir().unwrap();

  let package_source = (&pwd).join("package");
  if !package_source.exists() {
    return Err(Error::msg(format!(
      "{:?} is not existed,please create this folder",
      &package_source
    )));
  }
  if !package_source.is_dir() {
    return Err(Error::msg(format!(
      "{:?} is not a folder,please create this folder",
      &package_source
    )));
  }

  // skip copy libs
  if !args.skip_libs {
    let dist_source = (&pwd).join(&args.dist);

    if !dist_source.is_dir() {
      return Err(Error::msg(format!(
        "{:?} is not a folder,please confirm your dist path.",
        &package_source
      )));
    }

    let is_exist = fs::read_dir(&dist_source)
      .unwrap()
      .peekable()
      .peek()
      .is_some();

    if !is_exist {
      return Err(Error::msg(format!(
        "{:?} is empty,please run build before artifact.",
        &package_source
      )));
    }

    // clean the folder before we copy it
    check_and_clean_file_or_dir!((&package_source).join("libs"));

    // copy dist
    let mut op = CopyOptions::new();
    op.overwrite = true;
    op.copy_inside = true;
    fs_extra::dir::copy((&pwd).join(&args.dist), (&package_source).join("libs"), &op).unwrap();
  }

  let package_path = PathBuf::from(&pwd).join(format!("{}.har", args.name));

  generate_har(package_path, package_source);
  Ok(())
}
