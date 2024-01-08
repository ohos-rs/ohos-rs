use crate::build::Context;
use crate::create_dist_dir;

/// 构建前初始化工作，包括获取当前运行环境等。
pub fn prepare(ctx: &mut Context) -> Result<(), String> {
  let args = super::BUILD_ARGS.read().unwrap();
  ctx.pwd = std::env::current_dir().unwrap();

  ctx.mode = "debug";
  if args.release {
    ctx.mode = "release";
  }

  ctx.init_args = vec!["+nightly", "build", "-Z", "build-std"];

  if args.release {
    let _ = &ctx.init_args.push("--release");
  }

  // 创建目标文件夹
  ctx.dist = ctx.pwd.join(&args.dir);
  create_dist_dir!(&ctx.dist);

  Ok(())
}
