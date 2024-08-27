use crate::check_and_clean_file_or_dir;
use crate::util::Arch;
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Package;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

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
  // 当前构建项目的产物地址 用于支持cargo workspace的构建
  pub cargo_build_target_dir: Option<Utf8PathBuf>,
  // ndk 路径
  pub ndk: String,
  // 所有产物的文件路径 避免重复获取
  #[allow(dead_code)]
  pub dist_files: Vec<PathBuf>,
  pub template: Option<Template>,

  pub copy_static: bool,
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

  [Arch::ARM64, Arch::ARM32, Arch::X86_64]
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

      run::build(&cargo_args, &ctx, &arch)?;
      Ok(())
    })
    .collect::<anyhow::Result<Vec<_>>>()?;

  ts::generate_d_ts_file(&ctx)?;
  Ok(())
}
