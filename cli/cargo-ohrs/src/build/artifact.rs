use std::path::{Path, PathBuf};

use cargo_metadata::{camino::Utf8PathBuf, Artifact, BuildScript};

fn is_rust_intermediate_lib(path: &Utf8PathBuf) -> bool {
  let path_str = path.to_string();
  path_str.contains("/target/") && path_str.contains("/deps/")
}

pub fn resolve_dependence_library(
  script: BuildScript,
  ndk: String,
  hos_ndk: String,
) -> Option<Vec<PathBuf>> {
  let ohos_sysroot = Path::new(&ndk)
    .join("native")
    .join("sysroot")
    .join("usr")
    .join("lib");
  let hms_sysroot = Path::new(&hos_ndk)
    .join("native")
    .join("sysroot")
    .join("usr")
    .join("lib");

  if !script.linked_libs.is_empty() && !script.linked_paths.is_empty() {
    let libs = script
      .linked_libs
      .iter()
      .filter_map(|i| {
        let library_name = i.as_str();
        if library_name.starts_with("dylib=") {
          return Some(format!("lib{}.so", &library_name[6..]));
        }
        return Some(format!("lib{}.so", library_name));
      })
      .collect::<Vec<String>>();

    let p = script
      .linked_paths
      .iter()
      .filter_map(|i| {
        let item_path = i.as_str();
        let normalized_path = item_path.strip_prefix("native=").unwrap_or(item_path);
        let path = Path::new(normalized_path);
        // Ignore sysroot lib
        if path.starts_with(&ohos_sysroot) {
          return None;
        }
        if !hos_ndk.is_empty() && path.starts_with(&hms_sysroot) {
          return None;
        }
        if item_path.starts_with("native=") {
          return Some(
            PathBuf::from(&item_path[7..])
              .canonicalize()
              .expect(&format!("Convert {} to absolute path failed.", item_path)),
          );
        }
        if is_rust_intermediate_lib(&i) {
          return None;
        }
        let current_path = i
          .canonicalize()
          .expect(&format!("Convert {} to absolute path failed.", i.as_str()));
        if !i.is_dir() && !i.is_file() {
          println!("Note: {} is not a dir or file.", i.as_str());
          return None;
        }
        Some(current_path)
      })
      .collect::<Vec<_>>();

    let mut ret: Vec<PathBuf> = Vec::new();

    p.iter().fold(&mut ret, |current_ret, i| {
      let item_p = libs.iter().map(|l| return i.join(l)).collect::<Vec<_>>();
      current_ret.extend(item_p);
      current_ret
    });
    return Some(ret);
  }
  None
}

pub fn resolve_artifact_library(target: Artifact) -> Option<Vec<PathBuf>> {
  Some(
    target
      .filenames
      .iter()
      .filter_map(|i| {
        // Avoid final target having the same package name as crate
        // For example: build reqwest
        // Support build exec, but ignore it
        if let Some(ext) = i.extension() {
          if (ext == "so" || ext == "a") && !is_rust_intermediate_lib(i) {
            return Some(i);
          }
          return None;
        }
        None
      })
      .map(|i| i.canonicalize().expect("Convert to absolute path failed."))
      .collect::<Vec<_>>(),
  )
}
