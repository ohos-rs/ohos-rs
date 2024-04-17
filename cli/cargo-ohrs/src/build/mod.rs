use crate::arg::BuildArg;
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Package;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

mod artifact;
mod prepare;
mod run;
mod strip;
mod ts;

// 全局状态，在调用的时候必须重置为当前指
pub(crate) static BUILD_ARGS: Lazy<RwLock<BuildArg>> = Lazy::new(|| RwLock::default());

/// 构建产物目标
#[derive(Debug, Clone, Copy)]
pub struct Architecture<'a> {
  arch: &'a str,
  target: &'a str,
  #[allow(dead_code)]
  platform: &'a str,
}

impl<'a> Architecture<'a> {
  fn new(arch: &'a str, target: &'a str, platform: &'a str) -> Self {
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
pub fn build() {
  let ctx = Arc::new(RwLock::new(Context::default()));

  prepare::prepare(ctx.clone()).unwrap();

  let arm64 = Architecture::new("arm64-v8a", "aarch64-unknown-linux-ohos", "arm64");
  let arm = Architecture::new("armeabi-v7a", "armv7-unknown-linux-ohos", "arm");
  let x86 = Architecture::new("x86_64", "x86_64-unknown-linux-ohos", "x86_64");

  [arm64, arm, x86].map(|arch| {
    run::build(ctx.clone(), &arch);
    artifact::copy_artifact(ctx.clone(), &arch);
  });

  let mut threads = vec![];

  let ts_ctx = ctx.clone();
  threads.push(thread::spawn(move || {
    ts::generate_d_ts_file(ts_ctx);
  }));

  strip::strip(ctx.clone(), &mut threads);

  for t in threads {
    t.join().unwrap();
  }
}
