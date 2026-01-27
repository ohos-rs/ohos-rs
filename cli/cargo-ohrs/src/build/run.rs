use crate::{build::Context, util::Arch, *};
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

  let mut ndk_path = format!("{}", &ctx.ndk);
  if ctx.bisheng {
    ndk_path = format!("{}/native/BiSheng", &ctx.hos_ndk);
  } else {
    ndk_path = format!("{}/native/llvm", &ctx.ndk);
  }

  let ran_path = format!("{}/bin/llvm-ranlib", ndk_path);
  let ar_path = format!("{}/bin/llvm-ar", ndk_path);
  let cc_path = format!("{}/bin/clang", ndk_path);
  let cxx_path = format!("{}/bin/clang++", ndk_path);
  let as_path = format!("{}/bin/llvm-as", ndk_path);
  let ld_path = format!("{}/bin/ld.lld", ndk_path);
  let strip_path = format!("{}/bin/llvm-strip", ndk_path);
  let obj_dump_path = format!("{}/bin/llvm-objdump", ndk_path);
  let obj_copy_path = format!("{}/bin/llvm-objcopy", ndk_path);
  let nm_path = format!("{}/bin/llvm-nm", ndk_path);
  let bin_path = format!("{}/bin", ndk_path);
  // For bindgen, you may need to change to builtin clang or clang++ etc. You can set LIBCLANG_PATH and CLANG_PATH
  let lib_path = format!("{}/lib", ndk_path);
  let std_lib = format!("CXXSTDLIB_{}", &arch.rust_link_target());
  let std_lib_type = String::from("c++");

  // The ctx.ndk path typically has spaces on Windows which `link-args` doesn't support.
  // Therefore we collect the args in an array and set them via multiple `link-arg` uses.
  let mut base_flags = vec![
    format!("--target={}", &arch.c_target()),
    format!("--sysroot={}/native/sysroot", &ctx.ndk),
    "-D__MUSL__".into(),
  ];

  let mut path = env::var("PATH").unwrap_or_default();
  // for windows, path need to use ; as split symbol
  // for unix, should use :
  #[cfg(target_os = "windows")]
  {
    path = format!("{};{}", &bin_path, &path);
  }
  #[cfg(not(target_os = "windows"))]
  {
    path = format!("{}:{}", &bin_path, &path);
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

  let prepare_env = HashMap::from([
    (linker_name.as_str(), &cc_path),
    ("LIBCLANG_PATH", &lib_path),
    ("CLANG_PATH", &cxx_path),
    (std_lib.as_str(), &std_lib_type),
    ("TARGET_CC", &cc_path),
    ("TARGET_CXX", &cxx_path),
    ("TARGET_RANLIB", &ran_path),
    ("TARGET_AR", &ar_path),
    ("TARGET_AS", &as_path),
    ("TARGET_LD", &ld_path),
    ("TARGET_STRIP", &strip_path),
    ("TARGET_OBJDUMP", &obj_dump_path),
    ("TARGET_OBJCOPY", &obj_copy_path),
    ("TARGET_NM", &nm_path),
    ("CARGO_ENCODED_RUSTFLAGS", &rust_flags),
    ("PATH", &path),
    ("TYPE_DEF_TMP_PATH", &tmp_path),
    // support opencv-rust
    ("OPENCV_CLANG_ARGS", &base_flags),
    ("DEP_ATOMIC", &builtins),
  ]);

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
              if let Some(lib) = resolve_dependence_library(script, ctx.ndk.clone()) {
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
