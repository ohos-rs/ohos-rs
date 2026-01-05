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

/// 构建命令执行时的上下文
#[derive(Debug, Clone, Default)]
pub struct Context<'a> {
  // 当前运行环境
  pub pwd: PathBuf,
  // 构建执行命令
  pub init_args: Vec<&'a str>,
  // 当前构建模式
  #[allow(dead_code)]
  pub mode: &'a str,
  // 目标产物路径
  pub dist: PathBuf,
  // 构建的信息
  pub package: Option<Package>,

  pub workspace_packages: Vec<Package>,
  // 当前构建项目的产物地址 用于支持cargo workspace的构建
  pub cargo_build_target_dir: Option<Utf8PathBuf>,
  // ohos_ndk 路径
  pub ndk: String,
  // hos_ndk 路径
  pub hos_ndk: String,
  // 所有产物的文件路径 避免重复获取
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
}

/// build逻辑
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

  // 解析 package 参数（从 -p 参数或 cargo_args 中的 -p）
  let package_filter = current_args.package.clone().or_else(|| {
    cargo_args
      .iter()
      .position(|arg| arg == "-p" || arg == "--package")
      .and_then(|idx| cargo_args.get(idx + 1).cloned())
  });

  let is_workspace = !ctx.workspace_packages.is_empty();
  let packages_to_build = if is_workspace {
    let all_packages = &ctx.workspace_packages;
    // 如果指定了 package 参数，只构建指定的包
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
    // 单包模式，如果指定了 package 参数，检查是否匹配
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

    if is_workspace {
      package_ctx.dist = ctx.dist.join(&pkg.name);
      crate::create_dist_dir!(package_ctx.dist.clone());
    }

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

    // 为每个包执行构建
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
