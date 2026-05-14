use crate::build::get_hos_sdk;
use crate::util::{
  append_hms_link_flags, apply_hms_include_env, apply_ohos_cmake_env, resolve_hms_paths,
  resolve_toolchain_paths, Arch,
};
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{exit, Command, Stdio};

pub fn run(
  arch: &Arch,
  ndk: String,
  args: Vec<String>,
  bisheng: bool,
  soname: Option<String>,
  build_target_name: Option<String>,
) -> anyhow::Result<()> {
  let linker_name = format!("CARGO_TARGET_{}_LINKER", &arch.rust_link_target());
  let hos_ndk = get_hos_sdk(&ndk).unwrap_or_default();

  let ndk_path = if bisheng {
    if hos_ndk.is_empty() || !std::path::Path::new(&hos_ndk).exists() {
      return Err(anyhow::Error::msg(
        "Failed to get HarmonyOS NDK path while --bisheng is enabled.",
      ));
    }
    std::path::Path::new(&hos_ndk)
      .join("native")
      .join("BiSheng")
      .to_string_lossy()
      .to_string()
  } else {
    std::path::Path::new(&ndk)
      .join("native")
      .join("llvm")
      .to_string_lossy()
      .to_string()
  };

  let toolchain = resolve_toolchain_paths(&ndk_path);
  let std_lib = format!("CXXSTDLIB_{}", &arch.rust_link_target());
  let std_lib_type = String::from("c++");
  let sysroot_path = std::path::Path::new(&ndk)
    .join("native")
    .join("sysroot")
    .to_string_lossy()
    .to_string();

  let mut base_flags = vec![
    format!("--target={}", &arch.c_target()),
    format!("--sysroot={}", sysroot_path),
    "-D__MUSL__".into(),
  ];

  let hms_paths = resolve_hms_paths(&hos_ndk, arch);
  append_hms_link_flags(&mut base_flags, &hms_paths);

  let mut path = env::var("PATH").unwrap_or(String::default());
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

  let preset_args =
    env::var("CARGO_RUSTFLAGS").unwrap_or(env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default());

  if arch.to_arch() == "armeabi-v7a" {
    base_flags.push("-march=armv7-a".into());
    base_flags.push("-mfloat-abi=softfp".into());
    base_flags.push("-mtune=generic-armv7-a".into());
    base_flags.push("-mthumb".into());
  }

  // Add SONAME linker flag if specified
  if let Some(ref soname) = soname {
    base_flags.push(format!("-Wl,-soname,{}", soname));
  }

  let mut rust_flags = base_flags
    .iter()
    .map(|f| format!("-Clink-arg={f}"))
    .collect::<Vec<_>>()
    .join("\x1f");

  if !preset_args.is_empty() {
    rust_flags = format!("{}\x1f{}", &rust_flags, &preset_args)
  }

  // for some package deps on atomic
  let builtins = String::from("clang_rt.builtins");
  let build_target_name = build_target_name.unwrap_or_else(|| String::from("entry"));

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
    (
      "NAPI_BUILD_TARGET_NAME".to_string(),
      build_target_name.clone(),
    ),
    ("OPENCV_CLANG_ARGS".to_string(), base_flags.clone()),
    ("DEP_ATOMIC".to_string(), builtins.clone()),
  ]);

  apply_hms_include_env(&mut prepare_env, &hms_paths, arch.rust_target());
  apply_ohos_cmake_env(&mut prepare_env, arch.rust_target(), &ndk, arch)?;

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
