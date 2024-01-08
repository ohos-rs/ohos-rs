use crate::build::Context;
use crate::{create_dist_dir, move_file};
use std::path::PathBuf;

/// 构建目标产物文件夹 & 复制目标文件
pub fn copy_artifact(ctx: &mut Context, target: &super::Architecture) {
  let args = super::BUILD_ARGS.read().unwrap();
  let mut compact_dir = "";

  if !args.compact {
    compact_dir = &target.arch;
  }

  let bin_dir = &ctx.dist.join(compact_dir);

  create_dist_dir!(bin_dir);

  if let Some(artifact) = &ctx.artifact {
    artifact.filenames.iter().for_each(|p| {
      let file = PathBuf::from(p);
      let dist: PathBuf;
      if !args.compact {
        dist = bin_dir.join(file.file_name().unwrap());
      } else {
        let file_name = file.file_stem().unwrap().to_str().unwrap();
        dist = bin_dir.join(format!("{}_{}.so", file_name, &target.platform));
      }
      move_file!(file, dist);
    });
  }
}
