use crate::check_and_clean_file_or_dir;
use crate::cli::Arch;
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Package;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

mod abort_tmp;
mod artifact;
mod prepare;
mod run;
mod ts;

/// 构建产物目标
#[derive(Debug, Clone, Copy)]
pub struct Architecture<'a> {
  arch: &'a str,
  target: &'a str,
  platform: Arch,
}

impl<'a> Architecture<'a> {
  fn new(arch: &'a str, target: &'a str, platform: Arch) -> Self {
    Architecture {
      arch,
      target,
      platform,
    }
  }
}

/// 构建命令执行时的上下文
#[derive(Debug, Clone, Default)]
pub struct Context<'a> {
  // 当前运行环境
  pub pwd: PathBuf,
  // 构建执行命令
  pub init_args: Vec<&'a str>,
  // 当前构建模式
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
  pub dist_files: Vec<PathBuf>,
}

/// build逻辑
pub fn build(args: crate::BuildArgs) {
  let ctx = Arc::new(RwLock::new(Context::default()));

  prepare::prepare(args.clone(), ctx.clone()).unwrap();

  let build_arch = args.arch.clone().unwrap_or(vec![
    crate::Arch::ARM64,
    crate::Arch::ARM32,
    crate::Arch::X86_64,
  ]);

  let aarch = Architecture::new("arm64-v8a", "aarch64-unknown-linux-ohos", Arch::ARM64);
  let arm = Architecture::new("armeabi-v7a", "armv7-unknown-linux-ohos", Arch::ARM32);
  let x64 = Architecture::new("x86_64", "x86_64-unknown-linux-ohos", Arch::X86_64);

  [aarch, arm, x64]
    .iter()
    .filter_map(|&i| {
      if build_arch.contains(&i.platform) {
        return Some(i);
      }
      None
    })
    .for_each(|arch| {
      let tmp_file = env::var("TYPE_DEF_TMP_PATH").expect("Get .d.ts tmp filed.");
      check_and_clean_file_or_dir!(PathBuf::from(&tmp_file));

      run::build(ctx.clone(), &arch);
    });

  let ts_ctx = ctx.clone();
  ts::generate_d_ts_file(ts_ctx);
}
