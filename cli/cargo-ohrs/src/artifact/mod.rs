use crate::arg::ArtifactArg;
use crate::artifact::tgz::generate_har;
use fs_extra::dir::CopyOptions;
use std::fs;
use std::{env, path::PathBuf};

mod tgz;

pub fn artifact(args: ArtifactArg) {
  let pwd = env::current_dir().unwrap();

  let package_source = (&pwd).join("package");
  if !package_source.exists() {
    println!(
      "{:?} is not existed,please create this folder",
      &package_source
    );
    return;
  }
  if !package_source.is_dir() {
    println!(
      "{:?} is not a folder,please create this folder",
      &package_source
    );
    return;
  }

  let dist_source = (&pwd).join(&args.dist);

  if !dist_source.is_dir() {
    println!(
      "{:?} is not a folder,please confirm your dist path.",
      &package_source
    );
    return;
  }

  let is_exist = fs::read_dir(&dist_source)
    .unwrap()
    .peekable()
    .peek()
    .is_some();

  if !is_exist {
    println!(
      "{:?} is empty,please run build before artifact.",
      &package_source
    );
    return;
  }

  println!("{:?}",(&pwd).join(&args.dist));
  println!("{:?}",(&package_source).join("lib"));
  // copy dist
  let mut op = CopyOptions::new();
  op.overwrite = true;
  fs_extra::dir::copy((&pwd).join(&args.dist), (&package_source).join("libs"), &op).unwrap();

  let package_path = PathBuf::from(&pwd).join(format!("{}.har", args.name));

  generate_har(package_path, package_source);
}
