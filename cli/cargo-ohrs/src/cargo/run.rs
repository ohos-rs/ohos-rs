use crate::util::Arch;
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{exit, Command, Stdio};

pub fn run(
  arch: &Arch,
  ohos_ndk: String,
  hos_ndk: String,
  args: Vec<&String>,
) -> anyhow::Result<()> {
  let linker_name = format!("CARGO_TARGET_{}_LINKER", &arch.rust_link_target());
  let mut ndk = format!("{}{}", &ohos_ndk, "/native/llvm");
  if hos_ndk.len() > 0 {
    ndk = format!("{}{}", &hos_ndk, "/native/BiSheng");
  }
  let ran_path = format!("{}/bin/llvm-ranlib", &ndk);
  let ar_path = format!("{}/bin/llvm-ar", &ndk);
  let cc_path = format!("{}/bin/clang", &ndk);
  let cxx_path = format!("{}/bin/clang++", &ndk);
  let as_path = format!("{}/bin/llvm-as", &ndk);
  let ld_path = format!("{}/bin/ld.lld", &ndk);
  let strip_path = format!("{}/bin/llvm-strip", &ndk);
  let obj_dump_path = format!("{}/bin/llvm-objdump", &ndk);
  let obj_copy_path = format!("{}/bin/llvm-objcopy", &ndk);
  let nm_path = format!("{}/bin/llvm-nm", &ndk);
  let bin_path = format!("{}/bin", &ndk);
  // for bindgen, you may need to change to builtin clang or clang++ etc. You can set LIBCLANG_PATH and CLANG_PATH
  // let lib_path = format!("{}/native/llvm/lib", &ndk);
  let mut rustflags = format!(
    "-Clink-args=-target {} --sysroot={}/native/sysroot -D__MUSL__",
    &arch.c_target(),
    &ndk
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

  let preset_args =
    env::var("CARGO_RUSTFLAGS").unwrap_or(env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default());

  if arch.to_arch() == "armeabi-v7a" {
    rustflags = format!(
      "{} -march=armv7-a -mfloat-abi=softfp -mtune=generic-armv7-a -mthumb",
      rustflags
    );
  }
  rustflags = format!("{} {}", rustflags, preset_args);

  let prepare_env = HashMap::from([
    (linker_name.as_str(), &cc_path),
    // ("LIBCLANG_PATH",&lib_path),
    // ("CLANG_PATH",&cc_path),
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

  let mut child = Command::new("cargo")
    .args(args)
    .envs(&prepare_env)
    .stdout(Stdio::piped())
    .spawn()?;

  if let Some(ref mut stdout) = child.stdout {
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
      let line = line?;
      println!("{}", line);
    }
  }
  if let Some(ref mut stderr) = child.stderr {
    let reader = BufReader::new(stderr);

    for line in reader.lines() {
      let line = line?;
      println!("{}", line);
    }
  }

  let status = child.wait()?;
  if !status.success() {
    exit(status.code().unwrap_or(-1))
  }

  Ok(())
}
