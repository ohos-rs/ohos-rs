use crate::build::{Architecture, Context};
use crate::{check_and_clean_file_or_dir, create_dist_dir, move_file};
use cargo_metadata::Message;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use std::sync::{Arc, RwLock};

use super::artifact::{resolve_artifact_library, resolve_dependence_library};

static TARGET: Lazy<HashMap<&str, (&str, &str)>> = Lazy::new(|| {
  HashMap::from([
    (
      "arm64-v8a",
      ("aarch64-linux-ohos", "AARCH64_UNKNOWN_LINUX_OHOS"),
    ),
    (
      "armeabi-v7a",
      ("arm-linux-ohos", "ARMV7_UNKNOWN_LINUX_OHOS"),
    ),
    ("x86_64", ("x86_64-linux-ohos", "X86_64_UNKNOWN_LINUX_OHOS")),
  ])
});

pub fn build(c: Arc<RwLock<Context>>, arch: &Architecture) {
  let ctx = c.read().unwrap();
  let t = TARGET.get(&arch.arch).unwrap();
  let pkg = ctx.package.as_ref().unwrap();

  let linker_name = format!("CARGO_TARGET_{}_LINKER", t.1);
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
  // let lib_path = format!("{}/native/llvm/lib", &ctx.ndk);
  let mut rustflags = format!(
    "-Clink-args=-target {} --sysroot={}/native/sysroot -D__MUSL__",
    t.0, &ctx.ndk
  );

  let mut path = env::var("PATH").unwrap();
  // for windows, path need to use ; as split symbol
  // for unix, should use :
  #[cfg(target_os = "windows")]
  {
    path = format!("{};{}", &path, &bin_path);
  }
  #[cfg(not(target_os = "windows"))]
  {
    path = format!("{}:{}", &path, &bin_path);
  }

  let args =
    env::var("CARGO_RUSTFLAGS").unwrap_or(env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default());

  if arch.arch == "armeabi-v7a" {
    rustflags = format!(
      "{} -march=armv7-a -mfloat-abi=softfp -mtune=generic-armv7-a -mthumb",
      rustflags
    );
  }
  rustflags = format!("{} {}", rustflags, args);

  let prepare_env = HashMap::from([
    (linker_name.as_str(), &cc_path),
    // ("LIBCLANG_PATH",&lib_path),
    // ("CLANG_PATH",&cc_path),
    ("CC", &cc_path),
    ("CXX", &cxx_path),
    ("RANLIB", &ran_path),
    ("AR", &ar_path),
    ("AS", &as_path),
    ("LD", &ld_path),
    ("STRIP", &strip_path),
    ("OBJDUMP", &obj_dump_path),
    ("OBJCOPY", &obj_copy_path),
    ("NM", &nm_path),
    ("CARGO_ENCODED_RUSTFLAGS", &rustflags),
    ("PATH", &path),
  ]);

  let mut args = ctx.init_args.clone();
  args.extend([
    "--target",
    &arch.target,
    "--message-format=json-render-diagnostics",
  ]);

  let mut artifact_files: Vec<PathBuf> = Vec::new();

  let mut child = Command::new("cargo")
    .args(args)
    .envs(&prepare_env)
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to execute command");

  if let Some(ref mut stdout) = child.stdout {
    let reader = BufReader::new(stdout);

    for message in cargo_metadata::Message::parse_stream(reader) {
      match message.unwrap() {
        Message::CompilerMessage(msg) => {
          println!("{:?}", msg);
        }
        // get final compiled library
        Message::CompilerArtifact(artifact) => {
          if let Some(p) = resolve_artifact_library(&pkg, artifact) {
            artifact_files.extend(p);
          }
        }
        Message::BuildScriptExecuted(script) => {
          if let Some(lib) = resolve_dependence_library(script) {
            artifact_files.extend(lib);
          }
        }
        Message::BuildFinished(finished) => match finished.success {
          true => {
            let bin_dir = &ctx.dist.join(&arch.arch);
            check_and_clean_file_or_dir!(bin_dir);
            create_dist_dir!(bin_dir);

            artifact_files.iter().for_each(|i| {
              let dist = bin_dir.join(i.file_name().unwrap());
              move_file!(i, dist);
            })
          }
          false => exit(-1),
        },
        _ => (), // Unknown message
      }
    }
  }
}
