use crate::build::{Context, Template};
use crate::create_dist_dir;
use anyhow::Error;
use cargo_metadata::MetadataCommand;
use sha2::{Digest, Sha256};
use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

/// 构建前初始化工作，包括获取当前运行环境等。  
pub fn prepare(args: &mut crate::BuildArgs, ctx: &mut Context) -> anyhow::Result<()> {
  ctx.pwd = env::current_dir()?;

  // set copy_static variable
  ctx.copy_static = args.copy_static;
  ctx.skip_libs = args.skip_libs;
  ctx.dts_cache = args.dts_cache;

  // 判断当前构建环境以及获取metadata信息
  let cargo_file = ctx.pwd.join("./Cargo.toml");
  let cargo_file_str = cargo_file.to_str().unwrap_or_default();
  if cargo_file.try_exists().is_err() {
    return Err(Error::msg(format!(
      "No crate found in manifest: {}.",
      cargo_file_str
    )));
  }

  let metadata = MetadataCommand::new()
    .no_deps()
    .manifest_path(&cargo_file)
    .exec()?;

  let pkg = metadata
    .packages
    .iter()
    .find(|p| {
      return p.manifest_path.eq(cargo_file_str);
    })
    .ok_or(Error::msg("Try to get package meta-info failed."))?;

  let toml_content: Option<Template> = pkg
    .metadata
    .get("template")
    .and_then(|v| serde_json::from_value(v.clone()).unwrap_or(None));

  ctx.template = toml_content;

  ctx.package = Some((*pkg).clone());
  ctx.cargo_build_target_dir = Some(metadata.target_directory);

  ctx.init_args = vec!["build"];

  if let Some(cargo_args) = &args.cargo_args {
    // release mode and --release arg should be ignored
    if args.release && !cargo_args.contains(&String::from("--release")) {
      ctx.init_args.push("--release");
    }
  }

  // 创建目标文件夹
  ctx.dist = ctx.pwd.join(&args.dist);
  create_dist_dir!(ctx.dist.clone());

  // 设置生成.d.ts tmp file路径的环境变量
  let tmp_dir = env::temp_dir();

  let mut hasher = Sha256::new();
  hasher.update(&pkg.manifest_path.as_str());
  let hash_result = hasher.finalize();
  let hash_hex = format!("{:x}", hash_result);
  let short_hash = &hash_hex[..8];
  let file_name = format!("{}-{}.napi_type_def", &pkg.name, short_hash);
  let final_file_name;

  if !ctx.dts_cache {
    let _ = fs_extra::file::remove(&file_name).is_err();
    final_file_name = format!(
      "{}_{}.tmp",
      &file_name,
      SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
    );
  } else {
    final_file_name = format!("{}.tmp", &file_name);
  }

  // 拼接完整的文件路径
  let file_path = PathBuf::from(tmp_dir).join(final_file_name);
  env::set_var(
    "TYPE_DEF_TMP_PATH",
    file_path
      .to_str()
      .ok_or(Error::msg("Try to set TYPE_DEF_TMP_PATH failed."))?,
  );
  ctx.tmp_ts_file_path = file_path;

  // 获取 ndk 环境变量配置
  let ndk = env::var("OHOS_NDK_HOME").map_err(|_| {
    Error::msg(
      "Failed to get the OHOS_NDK_HOME environment variable, please make sure you have set it.",
    )
  })?;
  ctx.ndk = ndk;

  Ok(())
}
