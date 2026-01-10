use crate::check_and_clean_file_or_dir;
use crate::util::Arch;
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Package;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::env;
use std::path::{Path, PathBuf};

mod abort_tmp;
mod artifact;
mod prepare;
mod run;
mod ts;

#[allow(unused_imports)]
pub use artifact::*;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Template {
  pub header: Option<String>,
}

/// Context for build command execution
#[derive(Debug, Clone, Default)]
pub struct Context<'a> {
  // Current working environment
  pub pwd: PathBuf,
  // Build execution command
  pub init_args: Vec<&'a str>,
  // Current build mode
  #[allow(dead_code)]
  pub mode: &'a str,
  // Target artifact path
  pub dist: PathBuf,
  // Build information
  pub package: Option<Package>,

  pub workspace_packages: Vec<Package>,
  // Current build project artifact path, used to support cargo workspace builds
  pub cargo_build_target_dir: Option<Utf8PathBuf>,
  // ohos_ndk path
  pub ndk: String,
  // hos_ndk path
  pub hos_ndk: String,
  // All artifact file paths to avoid duplicate retrieval
  #[allow(dead_code)]
  pub dist_files: Vec<PathBuf>,
  pub template: Option<Template>,

  pub copy_static: bool,
  pub tmp_ts_file_path: PathBuf,

  pub skip_libs: bool,
  pub dts_cache: bool,
  pub skip_check: bool,
  pub zigbuild: bool,
  pub bisheng: bool,
  pub skip_napi_check: bool,
}

/// Build logic
pub fn build(args: crate::BuildArgs) -> anyhow::Result<()> {
  let mut current_args = args.clone();
  let mut ctx = Context::default();

  prepare::prepare(&mut current_args, &mut ctx)?;

  let build_arch = current_args.arch.unwrap_or(vec![
    crate::Arch::ARM64,
    crate::Arch::ARM32,
    crate::Arch::X86_64,
  ]);

  let cargo_args = current_args.cargo_args.unwrap_or_default();

  // Parse package parameter (from -p argument or -p in cargo_args)
  let package_filter = current_args.package.clone().or_else(|| {
    cargo_args
      .iter()
      .position(|arg| arg == "-p" || arg == "--package")
      .and_then(|idx| cargo_args.get(idx + 1).cloned())
  });

  let is_workspace = !ctx.workspace_packages.is_empty();
  let packages_to_build = if is_workspace {
    let all_packages = &ctx.workspace_packages;
    // If package parameter is specified, only build the specified package
    if let Some(ref pkg_name) = package_filter {
      all_packages
        .iter()
        .find(|p| p.name == *pkg_name)
        .map(|p| std::slice::from_ref(p))
        .ok_or_else(|| anyhow::anyhow!("Package '{}' not found in workspace", pkg_name))?
    } else {
      all_packages
    }
  } else if let Some(ref pkg) = ctx.package {
    // Single package mode, if package parameter is specified, check if it matches
    if let Some(ref pkg_name) = package_filter {
      if pkg.name != *pkg_name {
        return Err(anyhow::anyhow!(
          "Package '{}' not found. Current package is '{}'",
          pkg_name,
          pkg.name
        ));
      }
    }
    std::slice::from_ref(pkg)
  } else {
    return Err(anyhow::anyhow!("No package to build"));
  };

  for pkg in packages_to_build {
    let mut package_ctx = ctx.clone();
    package_ctx.package = Some(pkg.clone());

    // If executing in workspace root, need to set dist for each package in their respective directories
    // If executing in a package directory, dist has already been correctly set in prepare
    if is_workspace && packages_to_build.len() > 1 {
      // Executing in workspace root, set dist for each package in their respective directories
      // Check if ctx.dist is under ctx.pwd (i.e., if it's in the root directory)
      if let Some(manifest_dir) = pkg.manifest_path.parent() {
        let manifest_dir_path = PathBuf::from(manifest_dir.as_str());
        // If the parent directory of ctx.dist is ctx.pwd, it means dist is in the root directory
        // Need to set dist for each package in their respective directories
        if ctx.dist.parent() == Some(&ctx.pwd) {
          // Extract dist name from ctx.dist
          let dist_name = ctx
            .dist
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("dist");
          package_ctx.dist = manifest_dir_path.join(dist_name);
          crate::create_dist_dir!(package_ctx.dist.clone());
        }
        // Otherwise, ctx.dist is already in a package directory, no need to modify
      }
    }
    // If executing in a package directory (packages_to_build.len() == 1), dist has already been correctly set in prepare, no need to modify

    let target_dir = current_args.target_dir.to_owned().unwrap_or(
      package_ctx
        .cargo_build_target_dir
        .as_ref()
        .map(|d| d.to_string())
        .unwrap_or_default(),
    );

    let mut hasher = Sha256::new();
    hasher.update(&pkg.manifest_path.as_str());
    let hash_result = hasher.finalize();
    let hash_hex = format!("{:x}", hash_result);
    let short_hash = &hash_hex[..8];

    let mut tmp_full_path = PathBuf::from(&target_dir)
      .join("ohos-rs")
      .join(format!("{}-{}", &pkg.name, short_hash));

    if !package_ctx.dts_cache {
      let _ = fs_extra::file::remove(&tmp_full_path).is_err();
      tmp_full_path = PathBuf::from(format!(
        "{}_{}",
        tmp_full_path.to_str().unwrap_or_default(),
        std::time::SystemTime::now()
          .duration_since(std::time::SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_millis()
          .to_string()
      ));
    }

    fs_extra::dir::create_all(&tmp_full_path, false)?;
    package_ctx.tmp_ts_file_path = tmp_full_path.clone();

    env::set_var(
      "NAPI_TYPE_DEF_TMP_FOLDER",
      tmp_full_path.to_str().unwrap_or_default(),
    );

    // Execute build for each package
    [Arch::ARM64, Arch::ARM32, Arch::X86_64, Arch::LoongArch64]
      .iter()
      .filter_map(|&i| {
        if build_arch.contains(&i) {
          return Some(i);
        }
        None
      })
      .map(|arch| -> anyhow::Result<()> {
        let tmp_file_env = env::var("TYPE_DEF_TMP_PATH");
        if let Ok(tmp_file) = tmp_file_env {
          check_and_clean_file_or_dir!(PathBuf::from(&tmp_file));
        }

        run::build(&cargo_args, &package_ctx, &arch)?;
        Ok(())
      })
      .collect::<anyhow::Result<Vec<_>>>()?;

    ts::generate_d_ts_file(&package_ctx)?;
  }

  Ok(())
}

pub(crate) fn get_hos_sdk(ohos_ndk: &str) -> Option<String> {
  let mut hos_ndk = None;
  if let Some(root) = Path::new(ohos_ndk).parent() {
    if let Some(path) = root.join("hms").to_str() {
      hos_ndk = Some(path.to_string());
    }
  }
  if hos_ndk.is_none() {
    if let Ok(ndk) = env::var("HOS_NDK_HOME") {
      hos_ndk = Some(ndk);
    }
  }
  if hos_ndk.is_none() {
    println!("Currently use OpenHarmony SDK Compiler, Because Failed to get the HarmonyOS NDK.");
  }
  hos_ndk
}
