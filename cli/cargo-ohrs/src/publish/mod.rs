mod tgz;

use std::process::Command;
use crate::arg::PublishArg;

pub fn publish(arg: PublishArg) {
    Command::new("ohrs").args(["build","--release"]).spawn().unwrap();
    Command::new("ohpm").args(["publish"]).spawn().unwrap();
}

