use crate::util::Arch;
use anyhow::Error;
use cargo_metadata::{MetadataCommand, Package};
use std::{env, str::FromStr};

mod run;

/// Validate SONAME: check if it contains version number (e.g., ".so.1")
fn validate_soname(soname: &str) -> anyhow::Result<()> {
  // Check if SONAME contains version number pattern: .so. followed by digits
  if soname.contains(".so.") {
    // Extract the part after ".so."
    if let Some(part_after_so) = soname.split(".so.").nth(1) {
      // Check if it starts with a digit (version number)
      if part_after_so
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_digit())
      {
        return Err(Error::msg(format!(
          "SONAME format with version number is not supported: '{}'. Please use format like 'libxx.so' or 'xx'.",
          soname
        )));
      }
    }
  }
  Ok(())
}

/// Normalize SONAME: if only base name is provided (e.g., "xx"), convert to "libxx.so"
/// If already in full format (e.g., "libxx.so"), keep as is
/// Version numbers (e.g., "libxx.so.1") are not supported
fn normalize_soname(soname: &str) -> anyhow::Result<String> {
  // Validate SONAME format first
  validate_soname(soname)?;

  // If it already starts with "lib" and contains ".so", return as is
  if soname.starts_with("lib") && soname.contains(".so") {
    return Ok(soname.to_string());
  }
  // If it ends with ".so" but doesn't start with "lib", add "lib" prefix
  if soname.ends_with(".so") {
    return Ok(format!("lib{}", soname));
  }
  // Otherwise, add "lib" prefix and ".so" suffix
  Ok(format!("lib{}.so", soname))
}

fn get_workspace_packages() -> anyhow::Result<Vec<Package>> {
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
    let all_packages: Vec<Package> = metadata
      .workspace_members
      .iter()
      .filter_map(|member_id| {
        metadata
          .packages
          .iter()
          .find(|p| &p.id == member_id)
          .cloned()
      })
      .collect();

    // Check if current directory is within a package directory
    // If so, only return that package; otherwise return all packages
    let current_pkg = all_packages.iter().find(|p| {
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
      Ok(all_packages)
    }
  } else {
    Ok(vec![])
  }
}

pub fn cargo(args: crate::CargoArgs) -> anyhow::Result<()> {
  let ohos_ndk = env::var("OHOS_NDK_HOME").map_err(|_| {
    Error::msg(
      "Failed to get the OHOS_NDK_HOME environment variable, please make sure you have set it.",
    )
  })?;
  if args.args.len() < 1 {
    return Err(Error::msg(
      "You don't provide any command for current command.",
    ));
  }

  let (command, rest_args) = args.args.split_at(1);
  let mut target_arch = args.arch.unwrap_or(vec![Arch::ARM64]);
  let mut target_arg = None;

  // Parse package parameter (from -p argument or -p in args)
  let package_filter = args.package.clone().or_else(|| {
    rest_args
      .iter()
      .position(|arg| arg == "-p" || arg == "--package")
      .and_then(|idx| rest_args.get(idx + 1).cloned())
  });

  let mut iter = rest_args.iter().peekable();
  while let Some(arg) = iter.next() {
    if arg == "--target" {
      if let Some(&next_arg) = iter.peek() {
        target_arg = Some(next_arg.to_string());
        break;
      }
    }
  }

  // if disable-target is true, just run once.
  if args.disable_target {
    if let Some(t) = target_arg {
      let all_targets = [
        Arch::ARM32.rust_target(),
        Arch::ARM64.rust_target(),
        Arch::X86_64.rust_target(),
      ];
      let ret = all_targets.iter().find(|&&x| x == t.as_str());
      if let Some(r) = ret {
        let arch = Arch::from_str(r).map_err(|e| Error::msg(e))?;
        target_arch = vec![arch];
      } else {
        return Err(Error::msg("Only support ohos target"));
      }
    } else {
      return Err(Error::msg(
        "You don't provide any target for current command.",
      ));
    }
  }

  //
  let workspace_packages = get_workspace_packages()?;
  let is_workspace = workspace_packages.len() > 1;

  // If in workspace mode, execute command for each package separately
  if is_workspace {
    // If package parameter is specified, only process the specified package
    let packages_to_process: Vec<&Package> = if let Some(ref pkg_name) = package_filter {
      workspace_packages
        .iter()
        .filter(|p| p.name == *pkg_name)
        .collect()
    } else {
      workspace_packages.iter().collect()
    };

    if packages_to_process.is_empty() {
      if let Some(ref pkg_name) = package_filter {
        return Err(Error::msg(format!(
          "Package '{}' not found in workspace",
          pkg_name
        )));
      }
    }

    for pkg in packages_to_process {
      println!("Running cargo command for package: {}", pkg.name);

      target_arch
        .iter()
        .map(|arch| {
          let mut all_args: Vec<String> = match arch.to_arch() {
            "loongarch64" => vec!["+nightly".to_string()],
            _ => Vec::new(),
          };

          all_args.push(command[0].clone());
          // If rest_args doesn't have -p argument, add it
          if !rest_args
            .iter()
            .any(|arg| arg == "-p" || arg == "--package")
          {
            // Use package@version format to avoid ambiguity when there are multiple packages with the same name
            let package_spec = format!("{}@{}", pkg.name, pkg.version);
            all_args.push("-p".to_string());
            all_args.push(package_spec);
          }

          if !args.disable_target {
            all_args.extend(["--target".to_string(), arch.rust_target().to_string()]);
          }

          if arch.to_arch() == "loongarch64" {
            all_args.extend(["-Z".to_string(), "build-std".to_string()]);
          }

          all_args.extend(rest_args.iter().cloned());

          let normalized_soname = if let Some(ref s) = args.soname {
            Some(normalize_soname(s)?)
          } else {
            None
          };
          run::run(
            arch,
            ohos_ndk.clone(),
            all_args,
            args.bisheng,
            normalized_soname,
          )?;
          Ok(())
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    }
  } else {
    target_arch
      .iter()
      .map(|arch| {
        let mut all_args: Vec<String> = match arch.to_arch() {
          "loongarch64" => vec!["+nightly".to_string()],
          _ => Vec::new(),
        };

        all_args.push(command[0].clone());

        if !args.disable_target {
          all_args.extend(["--target".to_string(), arch.rust_target().to_string()]);
        }

        if arch.to_arch() == "loongarch64" {
          all_args.extend(["-Z".to_string(), "build-std".to_string()]);
        }

        all_args.extend(rest_args.iter().cloned());

        let normalized_soname = if let Some(ref s) = args.soname {
          Some(normalize_soname(s)?)
        } else {
          None
        };
        run::run(
          arch,
          ohos_ndk.clone(),
          all_args,
          args.bisheng,
          normalized_soname,
        )?;
        Ok(())
      })
      .collect::<anyhow::Result<Vec<_>>>()?;
  }

  Ok(())
}
