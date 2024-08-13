use cargo_metadata::{Artifact, BuildScript};
use std::path::PathBuf;

pub fn resolve_dependence_library(script: BuildScript, ndk: String) -> Option<Vec<PathBuf>> {
  let sysroot = format!("{ndk}/native/sysroot/usr/lib");

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
        if !i.is_dir() && !i.is_file() {
            println!("Note: {} is not a dir or file.", i.as_str());
            return None;
        }
        let item_path = i.as_str();
        // ignore sysroot lib
        if item_path.starts_with(&sysroot) {
          return None;
        }
        if item_path.starts_with("native=") {
          return Some(
            PathBuf::from(&item_path[7..])
              .canonicalize()
              .expect("Convert to absolute path failed."),
          );
        }
        return Some(i.canonicalize().expect("Convert to absolute path failed."));
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
        // avoid final target has the same package name with crate
        // for example: build reqwest
        // support build exec, but ignore it
        if let Some(ext) = i.extension() {
          if ext == "so" || ext == "a" {
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
