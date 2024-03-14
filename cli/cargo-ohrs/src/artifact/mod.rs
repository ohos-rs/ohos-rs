use crate::arg::ArtifactArg;
use crate::artifact::tgz::generate_har;
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

  let package_path = PathBuf::from(&pwd).join(format!("{}.har",args.name));

  generate_har(package_path, package_source);
}
