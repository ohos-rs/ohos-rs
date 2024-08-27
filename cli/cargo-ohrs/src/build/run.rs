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

pub fn build(cargo_args: &Vec<String>, ctx: &Context, arch: &Arch) -> anyhow::Result<()> {
  let linker_name = format!("CARGO_TARGET_{}_LINKER", &arch.rust_link_target());
  let ran_path = format!("{}/native/llvm/bin/llvm-ranlib", &ctx.ndk);
  let ar_path = format!("{}/native/llvm/bin/llvm-ar", &ctx.ndk);
  let cc_path = format!("{}/native/llvm/bin/clang", &ctx.ndk);
  let cxx_path = format!("{}/native/llvm/bin/clang++", &ctx.ndk);
  let as_path = format!("{}/native/llvm/bin/llvm-as", &ctx.ndk);
  let ld_path = format!("{}/native/llvm/bin/ld.lld", &ctx.ndk);
  let strip_path = format!("{}/native/llvm/bin/llvm-strip", &ctx.ndk);
  let obj_dump_path = format!("{}/native/llvm/bin/llvm-objdump", &ctx.ndk);
  let obj_copy_path = format!("{}/native/llvm/bin/llvm-objcopy", &ctx.ndk);
  let nm_path = format!("{}/native/llvm/bin/llvm-nm", &ctx.ndk);
  let bin_path = format!("{}/native/llvm/bin", &ctx.ndk);
  // for bindgen, you may need to change to builtin clang or clang++ etc. You can set LIBCLANG_PATH and CLANG_PATH
  let lib_path = format!("{}/native/llvm/lib", &ctx.ndk);
  let std_lib = format!("CXXSTDLIB_{}", &arch.rust_link_target());
  let std_lib_type = String::from("c++");

  let mut rustflags = format!(
    "-Clink-args=-target {} --sysroot={}/native/sysroot -D__MUSL__",
    &arch.c_target(),
    &ctx.ndk
  );

  let mut path = env::var("PATH").unwrap_or(String::default());
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
    rustflags = format!(
      "{} -march=armv7-a -mfloat-abi=softfp -mtune=generic-armv7-a -mthumb",
      rustflags
    );
  }
  rustflags = format!("{} {}", rustflags, args);

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
    ("CARGO_ENCODED_RUSTFLAGS", &rustflags),
    ("PATH", &path),
  ]);

  let mut args = ctx.init_args.clone();
  args.extend([
    "--target",
    &arch.rust_target(),
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
              if let Some(lib) = resolve_dependence_library(script, (&ctx.ndk).clone()) {
                artifact_files.extend(lib);
              }
            }
            Message::BuildFinished(finished) => match finished.success {
              true => {
                let bin_dir = &ctx.dist.join(&arch.to_arch());
                check_and_clean_file_or_dir!(bin_dir);
                create_dist_dir!(bin_dir);

                artifact_files
                  .iter()
                  .filter_map(|i| {
                    if ctx.copy_static {
                      return Some(i);
                    }
                    if let Some(ext) = i.extension() {
                      if ext == "a" {
                        return None;
                      }
                      return Some(i);
                    }
                    Some(i)
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
  Ok(())
}
