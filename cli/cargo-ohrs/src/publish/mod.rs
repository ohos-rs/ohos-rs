use crate::arg::PublishArg;
use std::process::Command;

pub fn publish(arg: PublishArg) {
  Command::new("ohrs")
    .args(["build", "--release"])
    .spawn()
    .unwrap();
  Command::new("ohpm").args(["publish"]).spawn().unwrap();
}
