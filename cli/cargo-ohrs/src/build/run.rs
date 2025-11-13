use crate::build::Context;
use crate::util::Arch;
use crate::{check_and_clean_file_or_dir, create_dist_dir, move_file};
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
  let ndk = if ctx.ohos_ndk.len() > 0 {
    format!("{}{}", &ctx.ohos_ndk, "/native/llvm")
  } else {
    format!("{}{}", &ctx.hos_ndk, "/native/BiSheng")
  };
  let ran_path = format!("{}/bin/llvm-ranlib", ndk);
  let ar_path = format!("{}/bin/llvm-ar", ndk);
  let cc_path = format!("{}/bin/clang", ndk);
  let cxx_path = format!("{}/bin/clang++", ndk);
  let as_path = format!("{}/bin/llvm-as", ndk);
  let ld_path = format!("{}/bin/ld.lld", ndk);
  let strip_path = format!("{}/bin/llvm-strip", ndk);
  let obj_dump_path = format!("{}/bin/llvm-objdump", ndk);
  let obj_copy_path = format!("{}/bin/llvm-objcopy", ndk);
  let nm_path = format!("{}/bin/llvm-nm", ndk);
  let bin_path = format!("{}/bin", ndk);
  // for bindgen, you may need to change to builtin clang or clang++ etc. You can set LIBCLANG_PATH and CLANG_PATH
  let lib_path = format!("{}/lib", ndk);
  let std_lib = format!("CXXSTDLIB_{}", &arch.rust_link_target());
  let std_lib_type = String::from("c++");

  // The ctx.ndk path typically has spaces on Windows which `link-args` doesn't support.
  // Therefore we collect the args in an array and set them via multiple `link-arg` uses.
  let mut base_flags = vec![
    format!("--target={}", &arch.c_target()),
    format!("--sysroot={}/native/sysroot", ndk),
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

  let args =
    env::var("CARGO_RUSTFLAGS").unwrap_or(env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default());

  if arch.to_arch() == "armeabi-v7a" {
    base_flags.push("-march=armv7-a".into());
    base_flags.push("-mfloat-abi=softfp".into());
    base_flags.push("-mtune=generic-armv7-a".into());
    base_flags.push("-mthumb".into());
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

  let mut args = ctx.init_args.clone();
  args.extend([
    "--target",
    arch.rust_target(),
    "--message-format=json-render-diagnostics",
  ]);

  // respect cli extra args
  args.extend(cargo_args.iter().map(|s| s.as_str()));

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
            // get final compiled library
            Message::CompilerArtifact(artifact) => {
              if let Some(p) = resolve_artifact_library(artifact) {
                artifact_files.extend(p);
              }
            }
            Message::BuildScriptExecuted(script) => {
              if let Some(lib) = resolve_dependence_library(script, ctx.ohos_ndk.clone()) {
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
