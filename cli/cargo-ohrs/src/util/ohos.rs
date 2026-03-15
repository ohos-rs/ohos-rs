use std::collections::HashMap;
use std::env;
use std::path::Path;

use super::Arch;

#[derive(Debug, Clone, Default)]
pub struct ToolchainPaths {
  pub ranlib: String,
  pub ar: String,
  pub cc: String,
  pub cxx: String,
  pub llvm_as: String,
  pub ld: String,
  pub strip: String,
  pub objdump: String,
  pub objcopy: String,
  pub nm: String,
  pub bin_dir: String,
  pub lib_dir: String,
}

#[derive(Debug, Clone, Default)]
pub struct HmsPaths {
  pub include: Option<String>,
  pub lib: Option<String>,
}

pub fn resolve_toolchain_paths(root: &str) -> ToolchainPaths {
  let bin_dir = Path::new(root).join("bin");
  let lib_dir = Path::new(root).join("lib");

  #[cfg(target_os = "windows")]
  let exe_suffix = ".exe";
  #[cfg(not(target_os = "windows"))]
  let exe_suffix = "";

  let to_string = |p: std::path::PathBuf| p.to_string_lossy().to_string();
  let tool = |name: &str| to_string(bin_dir.join(format!("{}{}", name, exe_suffix)));

  ToolchainPaths {
    ranlib: tool("llvm-ranlib"),
    ar: tool("llvm-ar"),
    cc: tool("clang"),
    cxx: tool("clang++"),
    llvm_as: tool("llvm-as"),
    ld: tool("ld.lld"),
    strip: tool("llvm-strip"),
    objdump: tool("llvm-objdump"),
    objcopy: tool("llvm-objcopy"),
    nm: tool("llvm-nm"),
    bin_dir: to_string(bin_dir),
    lib_dir: to_string(lib_dir),
  }
}

pub fn resolve_hms_paths(hos_ndk: &str, arch: &Arch) -> HmsPaths {
  if hos_ndk.is_empty() {
    return HmsPaths::default();
  }

  let include_path = Path::new(hos_ndk)
    .join("native")
    .join("sysroot")
    .join("usr")
    .join("include");
  let lib_path = Path::new(hos_ndk)
    .join("native")
    .join("sysroot")
    .join("usr")
    .join("lib")
    .join(arch.c_target());

  HmsPaths {
    include: include_path
      .exists()
      .then(|| include_path.to_string_lossy().to_string()),
    lib: lib_path
      .exists()
      .then(|| lib_path.to_string_lossy().to_string()),
  }
}

pub fn append_hms_link_flags(base_flags: &mut Vec<String>, hms_paths: &HmsPaths) {
  if let Some(lib) = hms_paths.lib.as_ref() {
    base_flags.push(format!("-L{}", lib));
    base_flags.push(format!("-Wl,-rpath-link,{}", lib));
  }
}

pub fn apply_hms_include_env(
  prepare_env: &mut HashMap<String, String>,
  hms_paths: &HmsPaths,
  rust_target: &str,
) {
  let Some(include) = hms_paths.include.as_ref() else {
    return;
  };

  let include_flag = format!("-I{}", include);

  append_env_with_flag(prepare_env, "TARGET_CFLAGS", &include_flag);
  append_env_with_flag(prepare_env, "TARGET_CXXFLAGS", &include_flag);

  let bindgen_target = rust_target.replace('-', "_");
  append_env_with_flag(
    prepare_env,
    &format!("BINDGEN_EXTRA_CLANG_ARGS_{}", bindgen_target),
    &include_flag,
  );
  append_env_with_flag(
    prepare_env,
    &format!("BINDGEN_EXTRA_CLANG_ARGS_{}", bindgen_target.to_uppercase()),
    &include_flag,
  );
}

#[cfg(target_os = "windows")]
pub fn apply_windows_ohos_cmake_env(
  prepare_env: &mut HashMap<String, String>,
  rust_target: &str,
  toolchain: &ToolchainPaths,
) {
  let target_key = format!("CMAKE_GENERATOR_{}", rust_target);
  let target_key_alt = format!("CMAKE_GENERATOR_{}", rust_target.replace('-', "_"));
  let has_generator = env::var(&target_key).is_ok()
    || env::var(&target_key_alt).is_ok()
    || env::var("TARGET_CMAKE_GENERATOR").is_ok()
    || env::var("CMAKE_GENERATOR").is_ok();

  if !has_generator {
    prepare_env.insert(String::from("TARGET_CMAKE_GENERATOR"), String::from("Ninja"));
  }

  if env::var("CC").is_err() {
    prepare_env.insert(String::from("CC"), toolchain.cc.clone());
  }
  if env::var("CXX").is_err() {
    prepare_env.insert(String::from("CXX"), toolchain.cxx.clone());
  }
  if env::var("ASM").is_err() {
    prepare_env.insert(String::from("ASM"), toolchain.cc.clone());
  }
}

#[cfg(not(target_os = "windows"))]
pub fn apply_windows_ohos_cmake_env(_: &mut HashMap<String, String>, _: &str, _: &ToolchainPaths) {}

fn append_env_with_flag(prepare_env: &mut HashMap<String, String>, key: &str, append: &str) {
  let current = env::var(key).unwrap_or_default();
  let merged = if current.is_empty() {
    append.to_string()
  } else {
    format!("{} {}", current, append)
  };
  prepare_env.insert(key.to_string(), merged);
}
