use crate::build::{get_hos_sdk, Context, Template};
use crate::create_dist_dir;
use anyhow::Error;
use cargo_metadata::{MetadataCommand, Package};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::SystemTime;
use std::{env, fs};
use version_compare::compare_to;
use version_compare::Cmp;

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

/// Pre-build initialization work, including getting the current runtime environment, etc.
pub fn prepare(args: &mut crate::BuildArgs, ctx: &mut Context) -> anyhow::Result<()> {
  ctx.pwd = env::current_dir()?;

  // set copy_static variable
  ctx.copy_static = args.copy_static;
  ctx.skip_libs = args.skip_libs;
  ctx.dts_cache = args.dts_cache;
  ctx.skip_check = args.skip_check;
  ctx.zigbuild = args.zigbuild;
  ctx.bisheng = args.bisheng;
  ctx.skip_napi_check = args.skip_napi_check;
  ctx.soname = if let Some(ref s) = args.soname {
    Some(normalize_soname(s)?)
  } else {
    None
  };

  let cargo_file = ctx.pwd.join("./Cargo.toml");
  let cargo_file_str = cargo_file.to_str().unwrap_or_default();
  if cargo_file.try_exists().is_err() {
    return Err(Error::msg(format!(
      "No Rust project found in path: {}.",
      cargo_file_str
    )));
  }

  let metadata = MetadataCommand::new()
    .no_deps()
    .manifest_path(&cargo_file)
    .exec()?;

  let is_workspace = !metadata.workspace_members.is_empty();

  let packages_to_build: Vec<&Package> = if is_workspace {
    let all_candidates: Vec<&Package> = metadata
      .workspace_members
      .iter()
      .filter_map(|member_id| metadata.packages.iter().find(|p| &p.id == member_id))
      .filter(|p| {
        ctx.skip_napi_check
          || p
            .dependencies
            .iter()
            .any(|dep| dep.name == "napi-derive-ohos")
      })
      .collect();

    // Check if current directory is within a package directory
    // If so, only build that package; otherwise build all packages
    let current_pkg = all_candidates.iter().find(|p| {
      if let Some(manifest_dir) = p.manifest_path.parent() {
        // Check if current directory is the same as or a subdirectory of the package directory
        let pwd_canonical = ctx.pwd.canonicalize().ok();
        let manifest_dir_path = PathBuf::from(manifest_dir.as_str());
        let manifest_dir_canonical = manifest_dir_path.canonicalize().ok();

        if let (Some(pwd), Some(md)) = (pwd_canonical, manifest_dir_canonical) {
          // Check if current directory starts with package directory (is same or subdirectory)
          pwd.starts_with(&md) || pwd == md
        } else {
          // Fallback: compare string paths if canonicalize fails
          let pwd_str = ctx.pwd.to_string_lossy();
          let manifest_dir_str = manifest_dir.as_str();
          pwd_str.starts_with(manifest_dir_str) || pwd_str == manifest_dir_str
        }
      } else {
        false
      }
    });

    if let Some(pkg) = current_pkg {
      if ctx.skip_napi_check
        || pkg
          .dependencies
          .iter()
          .any(|dep| dep.name == "napi-derive-ohos")
      {
        vec![*pkg]
      } else {
        vec![]
      }
    } else {
      // Build all packages (when running from workspace root)
      all_candidates
    }
  } else {
    let pkg = metadata
      .packages
      .iter()
      .find(|p| {
        return p.manifest_path.eq(cargo_file_str);
      })
      .ok_or(Error::msg("Try to get package meta-info failed."))?;
    if ctx.skip_napi_check
      || pkg
        .dependencies
        .iter()
        .any(|dep| dep.name == "napi-derive-ohos")
    {
      vec![pkg]
    } else {
      vec![]
    }
  };

  if packages_to_build.is_empty() {
    return Err(Error::msg("No package need to build."));
  }

  let pkg = packages_to_build[0];

  let toml_content: Option<Template> = pkg
    .metadata
    .get("template")
    .and_then(|v| serde_json::from_value(v.clone()).unwrap_or(None));

  // Check the version of the napi-ohos and napi-backend-ohos
  if !ctx.skip_check {
    let full_metadata = MetadataCommand::new().manifest_path(&cargo_file).exec()?;

    let napi_ohos_version = full_metadata
      .packages
      .iter()
      .find(|p| p.name == "napi-ohos")
      .and_then(|v| Some(v.version.to_string()))
      .ok_or(Error::msg(
        "Try to get the version of the napi-ohos failed.",
      ))?;
    let napi_backend_ohos_version = full_metadata
      .packages
      .iter()
      .find(|p| p.name == "napi-derive-ohos")
      .and_then(|v| Some(v.version.to_string()))
      .ok_or(Error::msg(
        "Try to get the version of the napi-derive-ohos failed.",
      ))?;

    let result = compare_to(&napi_ohos_version, "1.1.0", Cmp::Ge).unwrap_or(false);
    if !result {
      return Err(Error::msg(format!(
        r#"The version of the napi-ohos is not >= 1.1.0, please update the napi-ohos to >= 1.1.0, the current version is {}.
If you want to skip the check, you can set the skip_check to true: ohrs build --skip-check"#,
        &napi_ohos_version
      )));
    }

    let result = compare_to(&napi_backend_ohos_version, "1.1.0", Cmp::Ge).unwrap_or(false);
    if !result {
      return Err(Error::msg(format!(
        r#"The version of the napi-derive-ohos is not >= 1.1.0, please update the napi-derive-ohos to >= 1.1.0, the current version is {}.
If you want to skip the check, you can set the skip_check to true: ohrs build --skip-check"#,
        &napi_backend_ohos_version
      )));
    }
  }

  ctx.template = toml_content;

  ctx.package = Some((*pkg).clone());
  ctx.workspace_packages = packages_to_build.iter().map(|p| (*p).clone()).collect();
  ctx.cargo_build_target_dir = Some(metadata.target_directory.clone());

  ctx.init_args = if ctx.zigbuild {
    vec!["zigbuild"]
  } else {
    vec!["build"]
  };

  if let Some(cargo_args) = &args.cargo_args {
    // Release mode and --release arg should be ignored
    if args.release && !cargo_args.contains(&String::from("--release")) {
      ctx.init_args.push("--release");
    }
  }

  // Create target folder
  // In workspace mode, if current directory is in a package directory, dist should be in that package directory
  // Otherwise, dist is in the current working directory
  if is_workspace {
    // Check if current directory is in a package directory
    let current_pkg_dir = packages_to_build.iter().find_map(|p| {
      if let Some(manifest_dir) = p.manifest_path.parent() {
        let pwd_canonical = ctx.pwd.canonicalize().ok();
        let manifest_dir_path = PathBuf::from(manifest_dir.as_str());
        let manifest_dir_canonical = manifest_dir_path.canonicalize().ok();

        if let (Some(pwd), Some(md)) = (pwd_canonical, manifest_dir_canonical) {
          if pwd.starts_with(&md) || pwd == md {
            Some(manifest_dir_path)
          } else {
            None
          }
        } else {
          // Fallback: compare string paths if canonicalize fails
          let pwd_str = ctx.pwd.to_string_lossy();
          let manifest_dir_str = manifest_dir.as_str();
          if pwd_str.starts_with(manifest_dir_str) || pwd_str == manifest_dir_str {
            Some(manifest_dir_path)
          } else {
            None
          }
        }
      } else {
        None
      }
    });

    if let Some(pkg_dir) = current_pkg_dir {
      // Current directory is in a package directory, dist should be in that package directory
      ctx.dist = pkg_dir.join(&args.dist);
    } else {
      // Current directory is in workspace root, dist is in the root directory
      ctx.dist = ctx.pwd.join(&args.dist);
    }
  } else {
    // Single package mode, dist is in the current working directory
    ctx.dist = ctx.pwd.join(&args.dist);
  }
  create_dist_dir!(ctx.dist.clone());

  let target_dir = args
    .target_dir
    .to_owned()
    .unwrap_or(metadata.target_directory.clone().to_string());

  let mut hasher = Sha256::new();
  hasher.update(&pkg.manifest_path.as_str());
  let hash_result = hasher.finalize();
  let hash_hex = format!("{:x}", hash_result);
  let short_hash = &hash_hex[..8];

  let mut tmp_full_path = PathBuf::from(target_dir)
    .join("ohos-rs")
    .join(format!("{}-{}", &pkg.name, short_hash));

  env::set_var(
    "NAPI_TYPE_DEF_TMP_FOLDER",
    tmp_full_path.to_str().unwrap_or_default(),
  );

  if !ctx.dts_cache {
    let _ = fs_extra::file::remove(&tmp_full_path).is_err();
    tmp_full_path = PathBuf::from(format!(
      "{}_{}",
      tmp_full_path.to_str().unwrap_or_default(),
      SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
    ));
  }

  fs_extra::dir::create_all(&tmp_full_path, false)?;

  metadata.packages.iter().for_each(|p| {
    if p
      .dependencies
      .iter()
      .find(|name| name.name == "napi-derive-ohos")
      .is_some()
      && !fs::exists(&tmp_full_path).is_ok()
    {
      env::set_var(
        format!(
          "NAPI_FORCE_BUILD_{}",
          p.name.replace("-", "_").to_uppercase()
        ),
        SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_millis()
          .to_string(),
      );
    }
  });

  // Get ndk environment variable configuration
  let ohos_ndk = env::var("OHOS_NDK_HOME").map_err(|_| {
    Error::msg(
      "Failed to get the OHOS_NDK_HOME environment variable, please make sure you have set it.",
    )
  })?;
  ctx.hos_ndk = get_hos_sdk(&ohos_ndk).unwrap_or_default();
  ctx.ndk = ohos_ndk;

  Ok(())
}
