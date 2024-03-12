use crate::build::Context;
use crate::create_dist_dir;
use cargo_metadata::MetadataCommand;
use sha2::{Digest, Sha256};
use std::env;
use std::path::PathBuf;

/// 构建前初始化工作，包括获取当前运行环境等。  
pub fn prepare(ctx: &mut Context) -> Result<(), String> {
  let args = super::BUILD_ARGS.read().unwrap();
  ctx.pwd = env::current_dir().unwrap();

  // 判断当前构建环境以及获取metadata信息
  let cargo_file = ctx.pwd.join("./Cargo.toml");
  let cargo_file_str = cargo_file.to_str().unwrap();
  if cargo_file.try_exists().is_err() {
    return Err(format!("No crate found in manifest: {}.", cargo_file_str));
  }

  let metadata = MetadataCommand::new()
    .no_deps()
    .manifest_path(&cargo_file)
    .exec()
    .unwrap();

  let pkg = metadata
    .packages
    .iter()
    .find(|p| {
      return p.manifest_path.eq(cargo_file_str);
    })
    .expect("Unable to find crate to build.");

  ctx.package = Some((*pkg).clone());
  ctx.cargo_build_target_dir = Some(metadata.target_directory);

  ctx.mode = "debug";
  if args.release {
    ctx.mode = "release";
  }

  ctx.init_args = vec!["+nightly", "build", "-Z", "build-std"];

  if args.release {
    let _ = &ctx.init_args.push("--release");
  }

  // 创建目标文件夹
  ctx.dist = ctx.pwd.join(&args.dist);
  create_dist_dir!(&ctx.dist);

  // 设置生成.d.ts tmp file路径的环境变量
  let tmp_dir = env::temp_dir();

  let mut hasher = Sha256::new();
  hasher.update(&pkg.manifest_path.as_str());
  let hash_result = hasher.finalize();
  let hash_hex = format!("{:x}", hash_result);
  let short_hash = &hash_hex[..8];
  let file_name = format!("{}-{}.napi_type_def.tmp", &pkg.name, short_hash);

  // 拼接完整的文件路径
  let file_path = PathBuf::from(tmp_dir).join(file_name);
  env::set_var("TYPE_DEF_TMP_PATH", file_path.to_str().unwrap());

  // 获取 ndk 环境变量配置
  let ndk = env::var("OHOS_NDK_HOME").expect("Can't get OHOS_NDK_HOME.");
  ctx.ndk = ndk;

  Ok(())
}