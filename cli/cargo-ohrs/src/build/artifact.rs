use std::fs;
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

    if let Some(package) = &ctx.package {
        // 从target中解析构建产物
        let source = &ctx.cargo_build_target_dir.clone().unwrap().join(&target.target).join(&ctx.mode);
        let files: Vec<PathBuf> = fs::read_dir(source)
            .expect("Failed to read directory")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().map_or(false, |e| e == "so"))
            .collect();

        for file in files {
            let dist: PathBuf;
            if !args.compact {
                dist = bin_dir.join(file.file_name().unwrap());
            } else {
                let file_name = file.file_stem().unwrap().to_str().unwrap();
                dist = bin_dir.join(format!("{}_{}.so", file_name, &target.platform));
            }
            move_file!(file, dist);
        }
    }
}
