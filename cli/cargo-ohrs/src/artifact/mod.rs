use crate::artifact::tgz::generate_har;
use crate::check_and_clean_file_or_dir;
use anyhow::Error;
use cargo_metadata::{MetadataCommand, Package};
use fs_extra::dir::CopyOptions;
use std::fs;
use std::{env, path::PathBuf};

mod tgz;

fn get_napi_packages() -> anyhow::Result<Vec<Package>> {
  let pwd = env::current_dir()?;
  let cargo_file = pwd.join("./Cargo.toml");

  if cargo_file.try_exists().is_err() {
    return Ok(vec![]);
  }

  let metadata = MetadataCommand::new()
    .no_deps()
    .manifest_path(&cargo_file)
    .exec()?;

  let is_workspace = !metadata.workspace_members.is_empty();

  if is_workspace {
    let packages: Vec<Package> = metadata
      .workspace_members
      .iter()
      .filter_map(|member_id| {
        metadata
          .packages
          .iter()
          .find(|p| &p.id == member_id)
          .cloned()
      })
      .filter(|p| {
        p.dependencies
          .iter()
          .any(|dep| dep.name == "napi-derive-ohos")
      })
      .collect();
    Ok(packages)
  } else {
    let cargo_file_str = cargo_file.to_str().unwrap_or_default();
    if let Some(pkg) = metadata
      .packages
      .iter()
      .find(|p| p.manifest_path.eq(cargo_file_str))
    {
      if pkg
        .dependencies
        .iter()
        .any(|dep| dep.name == "napi-derive-ohos")
      {
        return Ok(vec![pkg.clone()]);
      }
    }
    Ok(vec![])
  }
}

pub fn artifact(args: crate::ArtifactArgs) -> anyhow::Result<()> {
  let pwd = env::current_dir().unwrap();

  let mut packages = get_napi_packages()?;
  let is_workspace = packages.len() > 1;

  if packages.is_empty() {
    return Err(Error::msg(
      "No package found with napi-derive-ohos dependency.",
    ));
  }

  // 如果指定了 package 参数，只处理指定的包
  if let Some(ref pkg_name) = args.package {
    packages.retain(|p| p.name == *pkg_name);
    if packages.is_empty() {
      return Err(Error::msg(format!(
        "Package '{}' not found or does not use napi-derive-ohos",
        pkg_name
      )));
    }
  }

  if is_workspace {
    for pkg in &packages {
      println!("Generating artifact for package: {}", pkg.name);

      let package_source = (&pwd).join("package").join(&pkg.name);
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
        let dist_source = (&pwd).join(&args.dist).join(&pkg.name);

        if !dist_source.is_dir() {
          return Err(Error::msg(format!(
            "{:?} is not a folder,please confirm your dist path.",
            &dist_source
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
            &dist_source
          )));
        }

        // clean the folder before we copy it
        check_and_clean_file_or_dir!((&package_source).join("libs"));

        // copy dist
        let mut op = CopyOptions::new();
        op.overwrite = true;
        op.copy_inside = true;
        fs_extra::dir::copy(&dist_source, (&package_source).join("libs"), &op)?;
      }

      let package_path = PathBuf::from(&pwd).join(format!("{}-{}.har", args.name, pkg.name));
      generate_har(package_path, package_source);
    }
  } else {
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
      fs_extra::dir::copy((&pwd).join(&args.dist), (&package_source).join("libs"), &op)?;
    }

    let package_path = PathBuf::from(&pwd).join(format!("{}.har", args.name));
    generate_har(package_path, package_source);
  }

  Ok(())
}
