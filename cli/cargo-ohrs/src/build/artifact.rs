use crate::build::Context;
use crate::{create_dist_dir, move_file};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 构建目标产物文件夹 & 复制目标文件
pub fn copy_artifact(c: Arc<RwLock<Context>>, target: &super::Architecture) {
  let mut ctx = c.write().unwrap();

  let bin_dir = &ctx.dist.join(&target.arch);

  create_dist_dir!(bin_dir);

  if let Some(_package) = &ctx.package {
    // 从target中解析构建产物
    let source = &ctx
      .cargo_build_target_dir
      .clone()
      .unwrap()
      .join(&target.target)
      .join(&ctx.mode);

    // support dynamic and static library 
    let files: Vec<PathBuf> = fs::read_dir(source)
      .expect("Failed to read directory")
      .filter_map(Result::ok)
      .map(|entry| entry.path())
      .filter(|path| path.is_file() && path.extension().map_or(false, |e| e == "so" || e == "a"))
      .collect();

    for file in files {
      let dist: PathBuf = bin_dir.join(file.file_name().unwrap());
      (*ctx).dist_files.push(dist.clone());
      move_file!(file, dist);
    }
  }
}
