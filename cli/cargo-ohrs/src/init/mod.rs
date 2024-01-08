use crate::{arg::InitArg, create_dist_dir};
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;

mod tmp;

use tmp::{
  ARM64_CPP_BUILD_SHELL, ARM64_C_BUILD_SHELL, ARM_CPP_BUILD_SHELL, ARM_C_BUILD_SHELL, BUILD_INIT,
  CARGO_CONFIG_TOML, CARGO_TOML, GIT_IGNORE, LIB_CODE, X86_64_CPP_BUILD_SHELL,
  X86_64_C_BUILD_SHELL,
};

macro_rules! create_project_file {
  ($strs: ident, $target: expr,$name: literal) => {{
    let mut tmp_file =
      std::fs::File::create($target).expect(format!("Create {} failed.", $name).as_str());
    let mut perms = tmp_file.metadata().unwrap().permissions();
    perms.set_mode(0o755);
    tmp_file.set_permissions(perms).unwrap();
    tmp_file
      .write_all($strs.as_bytes())
      .expect(format!("Write {} failed", $name).as_str());
    println!("Create {} succeed.", $name);
  }};
}

pub fn init(arg: InitArg) {
  let pwd = std::env::current_dir().expect("Can't get current work path");

  let target = pwd.join(&arg.name);

  if target.exists() == true {
    println!(
      "{} already existed. Please change your project name.",
      &arg.name
    );
    return;
  }

  create_dist_dir!(&target.join(".cargo"));
  create_dist_dir!(&target.join("scripts"));
  create_dist_dir!(&target.join("src"));

  create_project_file!(
    CARGO_CONFIG_TOML,
    &target.join(".cargo").join("config.toml"),
    "config.toml"
  );
  create_project_file!(
    ARM64_C_BUILD_SHELL,
    &target
      .join("scripts")
      .join("aarch64-unknown-linux-ohos-clang.sh"),
    "aarch64-unknown-linux-ohos-clang.sh"
  );
  create_project_file!(
    ARM64_CPP_BUILD_SHELL,
    &target
      .join("scripts")
      .join("aarch64-unknown-linux-ohos-clang++.sh"),
    "aarch64-unknown-linux-ohos-clang++.sh"
  );
  create_project_file!(
    ARM_C_BUILD_SHELL,
    &target
      .join("scripts")
      .join("armv7-unknown-linux-ohos-clang.sh"),
    "armv7-unknown-linux-ohos-clang.sh"
  );
  create_project_file!(
    ARM_CPP_BUILD_SHELL,
    &target
      .join("scripts")
      .join("armv7-unknown-linux-ohos-clang++.sh"),
    "armv7-unknown-linux-ohos-clang++.sh"
  );
  create_project_file!(
    X86_64_C_BUILD_SHELL,
    &target
      .join("scripts")
      .join("x86_64-unknown-linux-ohos-clang.sh"),
    "x86_64-unknown-linux-ohos-clang.sh"
  );
  create_project_file!(
    X86_64_CPP_BUILD_SHELL,
    &target
      .join("scripts")
      .join("x86_64-unknown-linux-ohos-clang++.sh"),
    "x86_64-unknown-linux-ohos-clang++.sh"
  );
  create_project_file!(LIB_CODE, &target.join("src").join("lib.rs"), "lib.rs");
  create_project_file!(BUILD_INIT, &target.join("build.rs"), "build.rs");
  create_project_file!(GIT_IGNORE, &target.join(".gitignore"), ".gitignore");

  let config = CARGO_TOML.replace("entry", &arg.name.as_str());
  create_project_file!(config, &target.join("Cargo.toml"), "Cargo.toml");
}
