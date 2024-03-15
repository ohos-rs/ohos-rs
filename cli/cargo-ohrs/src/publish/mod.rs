use crate::arg::PublishArg;
use std::process::Command;
use std::env;
use std::path::PathBuf;

pub fn publish(arg: PublishArg) {
  let pkg = arg.package.unwrap_or(String::from("package.har"));
  let mut pwd = env::current_dir().unwrap();
  let default_path = PathBuf::from(&pkg);
  if default_path.is_relative() {
    pwd = pwd.join(&pkg)
  }else {
    pwd = PathBuf::from(&pkg);
  }
  Command::new("ohrs")
    .args(["build", "--release"])
    .spawn()
    .unwrap();
  Command::new("ohpm").args(["publish"]).spawn().unwrap();
}
