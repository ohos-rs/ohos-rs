use crate::{arg::InitArg, create_dist_dir, create_project_file};

mod config;
mod package;
mod tmp;

use config::get_git_config;
use package::{CHANGELOG, LICENSE, PKG, README};
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

  if arg.package {
    let git_config = get_git_config();
    let pkg = arg.package_name.unwrap_or(arg.name.clone());
    create_dist_dir!(&target.join("package"));

    let readme = README.replace("@pkg", &pkg.as_str());
    create_project_file!(
      readme,
      &target.join("package").join("README.md"),
      "package/README.md"
    );

    let pkg_json5 = PKG
      .replace("@author", &git_config.author.as_str())
      .replace("@pkg", &pkg.as_str());
    create_project_file!(
      pkg_json5,
      &target.join("package").join("oh-package.json5"),
      "package/oh-package.json5"
    );

    let license = LICENSE.replace("@author", &git_config.author.as_str());
    create_project_file!(
      license,
      &target.join("package").join("LICENSE"),
      "package/LICENSE"
    );

    let export = format!(
      r#"export * from "lib{}.so""#,
      (&arg.name).replace("-", "_")
    );
    create_project_file!(
      export,
      &target.join("package").join("index.ets"),
      "package/index.ets"
    );

    create_project_file!(
      CHANGELOG,
      &target.join("package").join("CHANGELOG.md"),
      "package/CHANGELOG.md"
    );
  }
}
