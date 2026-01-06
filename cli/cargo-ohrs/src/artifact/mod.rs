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
    let all_candidates: Vec<Package> = metadata
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

    // Check if current directory is within a package directory
    // If so, only return that package; otherwise return all packages
    let current_pkg = all_candidates.iter().find(|p| {
      if let Some(manifest_dir) = p.manifest_path.parent() {
        let pwd_canonical = pwd.canonicalize().ok();
        let manifest_dir_path = std::path::PathBuf::from(manifest_dir.as_str());
        let manifest_dir_canonical = manifest_dir_path.canonicalize().ok();

        if let (Some(pwd_path), Some(md_path)) = (pwd_canonical, manifest_dir_canonical) {
          pwd_path.starts_with(&md_path) || pwd_path == md_path
        } else {
          // Fallback: compare string paths if canonicalize fails
          let pwd_str = pwd.to_string_lossy();
          let manifest_dir_str = manifest_dir.as_str();
          pwd_str.starts_with(manifest_dir_str) || pwd_str == manifest_dir_str
        }
      } else {
        false
      }
    });

    if let Some(pkg) = current_pkg {
      // Only return the package in the current directory
      Ok(vec![pkg.clone()])
    } else {
      // Return all packages (when running from workspace root)
      Ok(all_candidates)
    }
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

  // If package parameter is specified, only process the specified package
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

      // Determine package_source based on package manifest directory
      // In workspace mode, package folder should be in each package's directory
      let package_source = if let Some(manifest_dir) = pkg.manifest_path.parent() {
        let manifest_dir_path = PathBuf::from(manifest_dir.as_str());
        manifest_dir_path.join("package")
      } else {
        // Fallback to workspace root if manifest_dir is None
        (&pwd).join("package").join(&pkg.name)
      };

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

      // Skip copy libs
      if !args.skip_libs {
        // Determine dist_source based on package manifest directory
        // Check if dist is in package directory or workspace root
        let dist_source = if let Some(manifest_dir) = pkg.manifest_path.parent() {
          let manifest_dir_path = PathBuf::from(manifest_dir.as_str());
          let pkg_dist = manifest_dir_path.join(&args.dist);
          // If dist exists in package directory, use it; otherwise use workspace root
          if pkg_dist.exists() {
            pkg_dist
          } else {
            (&pwd).join(&args.dist).join(&pkg.name)
          }
        } else {
          (&pwd).join(&args.dist).join(&pkg.name)
        };

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

        // Clean the folder before we copy it
        check_and_clean_file_or_dir!((&package_source).join("libs"));

        // Copy dist
        let mut op = CopyOptions::new();
        op.overwrite = true;
        op.copy_inside = true;
        fs_extra::dir::copy(&dist_source, (&package_source).join("libs"), &op)?;
      }

      // Generate har file in the package's directory
      let package_path = if let Some(manifest_dir) = pkg.manifest_path.parent() {
        let manifest_dir_path = PathBuf::from(manifest_dir.as_str());
        manifest_dir_path.join(format!("{}-{}.har", args.name, pkg.name))
      } else {
        // Fallback to workspace root if manifest_dir is None
        PathBuf::from(&pwd).join(format!("{}-{}.har", args.name, pkg.name))
      };
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

      // Clean the folder before we copy it
      check_and_clean_file_or_dir!((&package_source).join("libs"));

      // Copy dist
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
