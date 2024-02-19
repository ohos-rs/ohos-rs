use crate::{arg::InitArg, create_dist_dir, create_project_file};

mod tmp;

use tmp::{BUILD_INIT, CARGO_TOML, GIT_IGNORE, LIB_CODE};

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

  create_dist_dir!(&target.join("src"));

  create_project_file!(LIB_CODE, &target.join("src").join("lib.rs"), "lib.rs");
  create_project_file!(BUILD_INIT, &target.join("build.rs"), "build.rs");
  create_project_file!(GIT_IGNORE, &target.join(".gitignore"), ".gitignore");

  let config = CARGO_TOML.replace("entry", &arg.name.as_str());
  create_project_file!(config, &target.join("Cargo.toml"), "Cargo.toml");
}
