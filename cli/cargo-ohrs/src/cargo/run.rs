use crate::build::get_hos_sdk;
use crate::util::Arch;
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{exit, Command, Stdio};

pub fn run(arch: &Arch, ndk: String, args: Vec<&String>, bisheng: bool) -> anyhow::Result<()> {
  let linker_name = format!("CARGO_TARGET_{}_LINKER", &arch.rust_link_target());

  let mut ndk = format!("{}{}", &ndk, "/native/llvm");
  if bisheng {
    let hos_ndk = get_hos_sdk(&ndk).unwrap();
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
  let mut base_flags = vec![
    format!("--target={}", &arch.c_target()),
    format!("--sysroot={}/native/sysroot", &ndk),
    "-D__MUSL__".into(),
  ];

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
    base_flags.push("-march=armv7-a".into());
    base_flags.push("-mfloat-abi=softfp".into());
    base_flags.push("-mtune=generic-armv7-a".into());
    base_flags.push("-mthumb".into());
  }

  let mut rust_flags = base_flags
    .iter()
    .map(|f| format!("-Clink-arg={f}"))
    .collect::<Vec<_>>()
    .join("\x1f");

  if !args.is_empty() {
    rust_flags = format!("{}\x1f{}", &rust_flags, &preset_args)
  }

  // for some package deps on atomic
  let builtins = String::from("clang_rt.builtins");

  let base_flags = base_flags.join(" ");

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
    ("CARGO_ENCODED_RUSTFLAGS", &rust_flags),
    ("PATH", &path),
    // support opencv-rust
    ("OPENCV_CLANG_ARGS", &base_flags),
    ("DEP_ATOMIC", &builtins),
  ]);

  let cmd_args: Vec<&&String> = args.iter().collect();

  let mut child = Command::new("cargo")
    .args(cmd_args)
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
