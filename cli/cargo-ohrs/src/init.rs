use std::fs;
use std::io::prelude::*;

use crate::tmps::{
  ARM64_CPP_BUILD_SHELL, ARM64_C_BUILD_SHELL, ARM_CPP_BUILD_SHELL, ARM_C_BUILD_SHELL, BUILD_INIT,
  CARGO_CONFIG_TOML, CARGO_TOML, LIB_CODE, X86_64_CPP_BUILD_SHELL, X86_64_C_BUILD_SHELL,
};

macro_rules! create_project_file {
  ($file_name: ident, $strs: ident, $target: expr,$name: literal) => {{
    let mut $file_name =
      std::fs::File::create($target).expect(format!("Create {} failed.", $name).as_str());
    $file_name
      .write_all($strs.as_bytes())
      .expect(format!("Write {} failed", $name).as_str());
    println!("Create ${} succeed.", $name);
  }};
}

macro_rules! create_project_dir {
  ($dir: literal, $target: expr) => {{
    let _ = std::fs::create_dir($target.join($dir))
      .expect(format!("Can't create {} dir.", $dir).as_str());
  }};
}

pub fn init(name: String) {
  let pwd = std::env::current_dir().expect("Can't get current work path");

  let target = pwd.join(&name);

  if target.exists() == true {
    println!("{} already existed.Please change your project name.", &name);
    return;
  }

  fs::create_dir(&target).expect("Can't create project path.");
  create_project_dir!(".cargo", &target);
  create_project_dir!("scripts", &target);
  create_project_dir!("src", &target);

  create_project_file!(
    config_file,
    CARGO_CONFIG_TOML,
    &target.join(".cargo").join("config.toml"),
    "config.toml"
  );
  create_project_file!(
    arm_c_shell,
    ARM64_C_BUILD_SHELL,
    &target
      .join("scripts")
      .join("aarch64-unknown-linux-ohos-clang.sh"),
    "aarch64-unknown-linux-ohos-clang.sh"
  );
  create_project_file!(
    arm_cpp_shell,
    ARM64_CPP_BUILD_SHELL,
    &target
      .join("scripts")
      .join("aarch64-unknown-linux-ohos-clang++.sh"),
    "aarch64-unknown-linux-ohos-clang++.sh"
  );
  create_project_file!(
    armv7_c_shell,
    ARM_C_BUILD_SHELL,
    &target
      .join("scripts")
      .join("armv7-unknown-linux-ohos-clang.sh"),
    "armv7-unknown-linux-ohos-clang.sh"
  );
  create_project_file!(
    armv7_cpp_shell,
    ARM_CPP_BUILD_SHELL,
    &target
      .join("scripts")
      .join("armv7-unknown-linux-ohos-clang++.sh"),
    "armv7-unknown-linux-ohos-clang++.sh"
  );
  create_project_file!(
    x86_c_shell,
    X86_64_C_BUILD_SHELL,
    &target
      .join("scripts")
      .join("x86_64-unknown-linux-ohos-clang.sh"),
    "x86_64-unknown-linux-ohos-clang.sh"
  );
  create_project_file!(
    x86_cpp_shell,
    X86_64_CPP_BUILD_SHELL,
    &target
      .join("scripts")
      .join("x86_64-unknown-linux-ohos-clang++.sh"),
    "x86_64-unknown-linux-ohos-clang++.sh"
  );
  create_project_file!(
    lib_code_file,
    LIB_CODE,
    &target.join("src").join("lib.rs"),
    "lib.rs"
  );
  create_project_file!(build_file, BUILD_INIT, &target.join("build.rs"), "build.rs");
  create_project_file!(
    toml_file,
    CARGO_TOML,
    &target.join("Cargo.toml"),
    "Cargo.toml"
  );
}
