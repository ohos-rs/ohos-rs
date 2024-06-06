use cargo_metadata::{Artifact, BuildScript, Package};
use std::path::PathBuf;

pub fn resolve_dependence_library(script: BuildScript) -> Option<Vec<PathBuf>> {
  if !script.linked_libs.is_empty() && !script.linked_paths.is_empty() {
    let libs = script
      .linked_libs
      .iter()
      .filter_map(|i| {
        let library_name = i.as_str();
        if library_name.starts_with("dylib=") {
          return Some(format!("lib{}.so", &library_name[6..]));
        }
        return None;
      })
      .collect::<Vec<String>>();

    let p = script
      .linked_paths
      .iter()
      .map(|i| {
        let item_path = i.as_str();
        if item_path.starts_with("native=") {
          return PathBuf::from(&item_path[7..])
            .canonicalize()
            .expect("Convert to absolute path failed.");
        }
        return i.canonicalize().expect("Convert to absolute path failed.");
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

pub fn resolve_artifact_library(pkg: &Package, target: Artifact) -> Option<Vec<PathBuf>> {
  if target.target.name == pkg.name {
    return Some(
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
    );
  }
  None
}
