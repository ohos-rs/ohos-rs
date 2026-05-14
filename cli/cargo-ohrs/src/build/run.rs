use crate::{
  build::Context,
  util::{
    append_hms_link_flags, apply_hms_include_env, apply_windows_ohos_cmake_env, resolve_hms_paths,
    resolve_toolchain_paths, Arch,
  },
  *,
};
use anyhow::Error;
use cargo_metadata::Message;
use std::collections::HashMap;
use std::env;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

use super::artifact::{resolve_artifact_library, resolve_dependence_library};

pub fn build(cargo_args: &[String], ctx: &Context, arch: &Arch) -> anyhow::Result<()> {
  let linker_name = format!("CARGO_TARGET_{}_LINKER", &arch.rust_link_target());

  let ndk_path = if ctx.bisheng {
    if ctx.hos_ndk.is_empty() || !std::path::Path::new(&ctx.hos_ndk).exists() {
      return Err(anyhow::Error::msg(
        "Failed to get HarmonyOS NDK path while --bisheng is enabled.",
      ));
    }
    std::path::Path::new(&ctx.hos_ndk)
      .join("native")
      .join("BiSheng")
      .to_string_lossy()
      .to_string()
  } else {
    std::path::Path::new(&ctx.ndk)
      .join("native")
      .join("llvm")
      .to_string_lossy()
      .to_string()
  };

  let toolchain = resolve_toolchain_paths(&ndk_path);
  let std_lib = format!("CXXSTDLIB_{}", &arch.rust_link_target());
  let std_lib_type = String::from("c++");
  let sysroot_path = std::path::Path::new(&ctx.ndk)
    .join("native")
    .join("sysroot")
    .to_string_lossy()
    .to_string();

  // The ctx.ndk path typically has spaces on Windows which `link-args` doesn't support.
  // Therefore we collect the args in an array and set them via multiple `link-arg` uses.
  let mut base_flags = vec![
    format!("--target={}", &arch.c_target()),
    format!("--sysroot={}", sysroot_path),
    "-D__MUSL__".into(),
  ];

  let hms_paths = resolve_hms_paths(&ctx.hos_ndk, arch);
  append_hms_link_flags(&mut base_flags, &hms_paths);

  let mut path = env::var("PATH").unwrap_or_default();
  // for windows, path need to use ; as split symbol
  // for unix, should use :
  #[cfg(target_os = "windows")]
  {
    path = format!("{};{}", &toolchain.bin_dir, &path);
  }
  #[cfg(not(target_os = "windows"))]
  {
    path = format!("{}:{}", &toolchain.bin_dir, &path);
  }

  // Respect cargo build flags and bash environment variables
  // 1. CARGO_RUSTFLAGS
  // 2. CARGO_ENCODED_RUSTFLAGS
  // 3. RUSTFLAGS
  // 4. ENCODED_RUSTFLAGS
  let args =
    env::var("CARGO_RUSTFLAGS").unwrap_or(env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or(
      env::var("RUSTFLAGS").unwrap_or(env::var("ENCODED_RUSTFLAGS").unwrap_or_default()),
    ));

  if arch.to_arch() == "armeabi-v7a" {
    base_flags.push("-march=armv7-a".into());
    base_flags.push("-mfloat-abi=softfp".into());
    base_flags.push("-mtune=generic-armv7-a".into());
    base_flags.push("-mthumb".into());
  }

  // Add SONAME linker flag if specified
  if let Some(ref soname) = ctx.soname {
    base_flags.push(format!("-Wl,-soname,{}", soname));
  }

  let tmp_path_str = ctx.tmp_ts_file_path.to_str().ok_or(Error::msg(
    "Try to set TYPE_DEF_TMP_PATH before build failed.",
  ))?;
  let tmp_path = String::from(tmp_path_str);

  // for some package deps on atomic
  let builtins = String::from("clang_rt.builtins");
  let build_target_name = ctx
    .build_target_name
    .as_deref()
    .unwrap_or("entry")
    .to_string();

  //let mut rust_flags = base_flags.join("\x1f");

  let mut rust_flags = base_flags
    .iter()
    .map(|f| format!("-Clink-arg={f}"))
    .collect::<Vec<_>>()
    .join("\x1f");

  if !args.is_empty() {
    rust_flags = format!("{}\x1f{}", &rust_flags, &args)
  }

  let base_flags = base_flags.join(" ");

  let mut prepare_env = HashMap::from([
    (linker_name.clone(), toolchain.cc.clone()),
    ("LIBCLANG_PATH".to_string(), toolchain.lib_dir.clone()),
    ("CLANG_PATH".to_string(), toolchain.cxx.clone()),
    (std_lib.clone(), std_lib_type.clone()),
    ("TARGET_CC".to_string(), toolchain.cc.clone()),
    ("TARGET_CXX".to_string(), toolchain.cxx.clone()),
    ("TARGET_RANLIB".to_string(), toolchain.ranlib.clone()),
    ("TARGET_AR".to_string(), toolchain.ar.clone()),
    ("TARGET_AS".to_string(), toolchain.llvm_as.clone()),
    ("TARGET_LD".to_string(), toolchain.ld.clone()),
    ("TARGET_STRIP".to_string(), toolchain.strip.clone()),
    ("TARGET_OBJDUMP".to_string(), toolchain.objdump.clone()),
    ("TARGET_OBJCOPY".to_string(), toolchain.objcopy.clone()),
    ("TARGET_NM".to_string(), toolchain.nm.clone()),
    ("CARGO_ENCODED_RUSTFLAGS".to_string(), rust_flags.clone()),
    ("PATH".to_string(), path.clone()),
    ("TYPE_DEF_TMP_PATH".to_string(), tmp_path.clone()),
    (
      "NAPI_BUILD_TARGET_NAME".to_string(),
      build_target_name.clone(),
    ),
    ("OPENCV_CLANG_ARGS".to_string(), base_flags.clone()),
    ("DEP_ATOMIC".to_string(), builtins.clone()),
  ]);

  apply_hms_include_env(&mut prepare_env, &hms_paths, arch.rust_target());
  apply_windows_ohos_cmake_env(&mut prepare_env, arch.rust_target(), &ctx.ndk, arch)?;

  let mut args: Vec<String> = match arch.to_arch() {
    "loongarch64" => {
      // loongarch64 need to use nightly rust and build-std which is tier3 stage
      let mut init_args = vec!["+nightly".to_string()];
      init_args.extend(ctx.init_args.iter().map(|s| s.to_string()));
      init_args.extend(["-Z".to_string(), "build-std".to_string()]);
      init_args
    }
    _ => ctx.init_args.iter().map(|s| s.to_string()).collect(),
  };

  args.extend([
    "--target".to_string(),
    arch.rust_target().to_string(),
    "--message-format=json-render-diagnostics".to_string(),
  ]);

  if let Some(ref pkg) = ctx.package {
    let has_package_arg = cargo_args
      .iter()
      .any(|arg| arg == "-p" || arg == "--package");

    if !has_package_arg && ctx.workspace_packages.len() > 1 {
      // Use package@version format to avoid ambiguity when there are multiple packages with the same name
      let package_spec = format!("{}@{}", pkg.name, pkg.version);
      args.push("-p".to_string());
      args.push(package_spec);
    }
  }

  // respect cli extra args
  args.extend(cargo_args.iter().cloned());

  let mut artifact_files: Vec<PathBuf> = Vec::new();

  let mut child = Command::new("cargo")
    .args(args)
    .envs(&prepare_env)
    .stdout(Stdio::piped())
    .spawn()?;

  if let Some(ref mut stdout) = child.stdout {
    let reader = BufReader::new(stdout);

    for message in cargo_metadata::Message::parse_stream(reader) {
      match message {
        Ok(m) => {
          match m {
            Message::CompilerMessage(msg) => {
              println!("{:?}", msg);
            }
            // Get final compiled library
            Message::CompilerArtifact(artifact) => {
              if let Some(p) = resolve_artifact_library(artifact) {
                artifact_files.extend(p);
              }
            }
            Message::BuildScriptExecuted(script) => {
              if let Some(lib) =
                resolve_dependence_library(script, ctx.ndk.clone(), ctx.hos_ndk.clone())
              {
                artifact_files.extend(lib);
              }
            }
            Message::BuildFinished(finished) => match finished.success {
              true => {
                if ctx.skip_libs {
                  return Ok(());
                }
                let bin_dir = &ctx.dist.join(arch.to_arch());
                check_and_clean_file_or_dir!(bin_dir);
                create_dist_dir!(bin_dir);

                artifact_files
                  .iter()
                  .filter(|i| {
                    if ctx.copy_static {
                      return true;
                    }
                    if let Some(ext) = i.extension() {
                      if ext == "a" {
                        return false;
                      }
                    }
                    true
                  })
                  .for_each(|i| {
                    if let Some(f) = i.file_name() {
                      let dist = bin_dir.join(f);
                      move_file!(i, dist);
                    }
                  })
              }
              false => exit(-1),
            },
            _ => (), // Unknown message
          }
        }
        Err(e) => {
          return Err(Error::new(e));
        }
      }
    }
  }

  let status = child.wait()?;
  if !status.success() {
    exit(status.code().unwrap_or(-1))
  }

  Ok(())
}
