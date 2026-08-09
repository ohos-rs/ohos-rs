use std::collections::{HashSet, VecDeque};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use cargo_metadata::{Artifact, BuildScript, PackageId};

use crate::util::Arch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLibrary {
  pub source: PathBuf,
  pub soname: String,
}

/// Collect usable native search directories without assuming every directory
/// advertised by a build script exists. Some native builds emit several
/// candidate layouts and rely on the linker to use whichever one is present.
pub fn resolve_link_search_paths(script: &BuildScript, excluded_roots: &[PathBuf]) -> Vec<PathBuf> {
  resolve_link_search_path_values(
    script.linked_paths.iter().map(|path| path.as_str()),
    excluded_roots,
  )
}

fn resolve_link_search_path_values<'a>(
  values: impl IntoIterator<Item = &'a str>,
  excluded_roots: &[PathBuf],
) -> Vec<PathBuf> {
  let excluded_roots = excluded_roots
    .iter()
    .map(|path| path.canonicalize().unwrap_or_else(|_| path.clone()))
    .collect::<Vec<_>>();
  let mut seen = HashSet::new();

  values
    .into_iter()
    .filter_map(|value| {
      let path = PathBuf::from(strip_link_search_kind(value));
      if !path.is_dir() {
        return None;
      }
      let path = path.canonicalize().ok()?;
      if excluded_roots.iter().any(|root| path.starts_with(root)) {
        return None;
      }
      seen.insert(path.clone()).then_some(path)
    })
    .collect()
}

fn strip_link_search_kind(value: &str) -> &str {
  let Some((kind, path)) = value.split_once('=') else {
    return value;
  };
  match kind {
    "native" | "dependency" | "crate" | "framework" | "all" => path,
    _ => value,
  }
}

pub fn resolve_artifact_search_paths(artifact: &Artifact) -> Vec<PathBuf> {
  let mut seen = HashSet::new();
  artifact
    .filenames
    .iter()
    .filter_map(|filename| filename.parent())
    .map(|path| path.as_std_path())
    .filter(|path| path.is_dir())
    .filter_map(|path| path.canonicalize().ok())
    .filter(|path| seen.insert(path.clone()))
    .collect()
}

pub fn resolve_artifact_library(artifact: &Artifact, package_id: &PackageId) -> Vec<PathBuf> {
  if &artifact.package_id != package_id {
    return Vec::new();
  }

  artifact
    .filenames
    .iter()
    .filter(|filename| matches!(filename.extension(), Some("so" | "a")))
    .map(|filename| filename.as_std_path())
    .filter(|filename| filename.is_file())
    .map(Path::to_path_buf)
    .collect()
}

pub fn ohos_system_library_paths(ndk: &Path, hos_ndk: Option<&Path>, arch: Arch) -> Vec<PathBuf> {
  let mut paths = vec![
    ndk
      .join("native")
      .join("sysroot")
      .join("usr")
      .join("lib")
      .join(arch.c_target()),
    ndk.join("native").join("sysroot").join("usr").join("lib"),
  ];
  if let Some(hos_ndk) = hos_ndk {
    paths.extend([
      hos_ndk
        .join("native")
        .join("sysroot")
        .join("usr")
        .join("lib")
        .join(arch.c_target()),
      hos_ndk
        .join("native")
        .join("sysroot")
        .join("usr")
        .join("lib"),
    ]);
  }
  deduplicate_paths(paths)
}

pub fn ohos_runtime_library_paths(ndk: &Path, arch: Arch) -> Vec<PathBuf> {
  let target_dir = ndk
    .join("native")
    .join("llvm")
    .join("lib")
    .join(arch.c_target());
  deduplicate_paths(vec![target_dir.clone(), target_dir.join("c++")])
}

/// Resolve the runtime closure from the final ELF artifacts. Cargo link
/// directives are intentionally used only as search hints: they also contain
/// static libraries, linker scripts, and libraries removed by `--as-needed`.
pub fn resolve_runtime_libraries(
  root_artifacts: &[PathBuf],
  dependency_search_paths: &[PathBuf],
  runtime_search_paths: &[PathBuf],
  system_search_paths: &[PathBuf],
  llvm_readobj: &Path,
) -> Result<Vec<RuntimeLibrary>> {
  resolve_runtime_libraries_with(
    root_artifacts,
    dependency_search_paths,
    runtime_search_paths,
    system_search_paths,
    |artifact| read_needed_libraries(llvm_readobj, artifact),
  )
}

fn resolve_runtime_libraries_with<F>(
  root_artifacts: &[PathBuf],
  dependency_search_paths: &[PathBuf],
  runtime_search_paths: &[PathBuf],
  system_search_paths: &[PathBuf],
  mut read_needed: F,
) -> Result<Vec<RuntimeLibrary>>
where
  F: FnMut(&Path) -> Result<Vec<String>>,
{
  let dependency_search_paths = deduplicate_paths(dependency_search_paths.to_vec());
  let runtime_search_paths = deduplicate_paths(runtime_search_paths.to_vec());
  let system_search_paths = deduplicate_paths(system_search_paths.to_vec());
  let mut pending = root_artifacts
    .iter()
    .filter(|path| path.extension() == Some(OsStr::new("so")))
    .cloned()
    .collect::<VecDeque<_>>();
  let mut scanned_files = HashSet::new();
  let mut handled_sonames = HashSet::new();
  let mut resolved = Vec::new();

  while let Some(requester) = pending.pop_front() {
    let requester_identity = requester
      .canonicalize()
      .unwrap_or_else(|_| requester.clone());
    if !scanned_files.insert(requester_identity) {
      continue;
    }

    for soname in read_needed(&requester)? {
      if !handled_sonames.insert(soname.clone()) {
        continue;
      }
      validate_soname(&soname)?;

      let requester_dir = requester.parent().into_iter();
      let package_candidate = requester_dir
        .chain(dependency_search_paths.iter().map(PathBuf::as_path))
        .find_map(|directory| find_library(directory, &soname));
      let runtime_candidate = runtime_search_paths
        .iter()
        .find_map(|directory| find_library(directory, &soname));

      if let Some(source) = package_candidate.or(runtime_candidate) {
        pending.push_back(source.clone());
        resolved.push(RuntimeLibrary { source, soname });
        continue;
      }

      if system_search_paths
        .iter()
        .any(|directory| find_library(directory, &soname).is_some())
      {
        continue;
      }

      bail!(
        "Unable to resolve runtime library '{}' required by {}",
        soname,
        requester.display()
      );
    }
  }

  Ok(resolved)
}

fn read_needed_libraries(llvm_readobj: &Path, artifact: &Path) -> Result<Vec<String>> {
  let output = Command::new(llvm_readobj)
    .arg("--needed-libs")
    .arg(artifact)
    .output()
    .with_context(|| {
      format!(
        "Failed to inspect runtime dependencies for {}",
        artifact.display()
      )
    })?;

  if !output.status.success() {
    bail!(
      "Failed to inspect runtime dependencies for {}: {}",
      artifact.display(),
      String::from_utf8_lossy(&output.stderr).trim()
    );
  }

  parse_needed_libraries(&String::from_utf8_lossy(&output.stdout)).with_context(|| {
    format!(
      "Failed to parse runtime dependencies for {}",
      artifact.display()
    )
  })
}

fn parse_needed_libraries(output: &str) -> Result<Vec<String>> {
  let mut in_needed_libraries = false;
  let mut found_section = false;
  let mut libraries = Vec::new();

  for line in output.lines() {
    let line = line.trim();
    if line == "NeededLibraries [" {
      in_needed_libraries = true;
      found_section = true;
      continue;
    }
    if in_needed_libraries && line == "]" {
      in_needed_libraries = false;
      continue;
    }
    if in_needed_libraries && !line.is_empty() {
      libraries.push(line.to_owned());
    }
  }

  if !found_section || in_needed_libraries {
    bail!("llvm-readobj output did not contain a complete NeededLibraries section");
  }

  Ok(libraries)
}

fn validate_soname(soname: &str) -> Result<()> {
  let path = Path::new(soname);
  if path.file_name() != Some(OsStr::new(soname)) {
    bail!("Invalid runtime library name reported by ELF: {soname}");
  }
  Ok(())
}

fn find_library(directory: &Path, soname: &str) -> Option<PathBuf> {
  let candidate = directory.join(soname);
  candidate.is_file().then_some(candidate)
}

fn deduplicate_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
  let mut seen = HashSet::new();
  paths
    .into_iter()
    .filter(|path| path.is_dir())
    .filter_map(|path| path.canonicalize().ok())
    .filter(|path| seen.insert(path.clone()))
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::sync::atomic::{AtomicU64, Ordering};

  static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(1);

  struct TestDir(PathBuf);

  impl TestDir {
    fn new() -> Self {
      let id = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
      let path = std::env::temp_dir().join(format!("ohrs-artifact-{}-{id}", std::process::id()));
      fs::create_dir_all(&path).unwrap();
      Self(path)
    }
  }

  impl Drop for TestDir {
    fn drop(&mut self) {
      let _ = fs::remove_dir_all(&self.0);
    }
  }

  #[test]
  fn missing_and_excluded_link_search_paths_are_ignored() {
    let temp = TestDir::new();
    let dependency = temp.0.join("dependency");
    let system = temp.0.join("system");
    fs::create_dir_all(&dependency).unwrap();
    fs::create_dir_all(&system).unwrap();

    let values = [
      format!("native={}", temp.0.join("missing").display()),
      format!("native={}", system.display()),
      format!("native={}", dependency.display()),
      format!("native={}", dependency.display()),
    ];
    let resolved = resolve_link_search_path_values(
      values.iter().map(String::as_str),
      std::slice::from_ref(&system),
    );

    assert_eq!(resolved, vec![dependency.canonicalize().unwrap()]);
  }

  #[test]
  fn boring_ssl_candidate_layouts_do_not_require_empty_directories() {
    let temp = TestDir::new();
    let build = temp.0.join("build");
    let ssl = build.join("ssl");
    fs::create_dir_all(&ssl).unwrap();

    let values = [
      format!("native={}", build.join("lib").display()),
      format!("native={}", build.join("crypto").display()),
      format!("native={}", ssl.display()),
      format!("native={}", build.display()),
    ];
    let resolved = resolve_link_search_path_values(values.iter().map(String::as_str), &[]);

    assert_eq!(
      resolved,
      vec![ssl.canonicalize().unwrap(), build.canonicalize().unwrap()]
    );
  }

  #[test]
  fn needed_libraries_are_parsed_from_llvm_readobj_output() {
    let output = r#"
File: libexample.so
NeededLibraries [
  libc++_shared.so
  libc.so
]
"#;

    assert_eq!(
      parse_needed_libraries(output).unwrap(),
      vec!["libc++_shared.so", "libc.so"]
    );
  }

  #[test]
  fn malformed_needed_libraries_output_is_rejected() {
    assert!(parse_needed_libraries("File: libexample.so").is_err());
    assert!(parse_needed_libraries("NeededLibraries [\nlibfoo.so").is_err());
  }

  #[test]
  fn sonames_cannot_escape_the_search_directory() {
    assert!(validate_soname("libexample.so").is_ok());
    assert!(validate_soname("../libexample.so").is_err());
    assert!(validate_soname("nested/libexample.so").is_err());
  }

  #[test]
  fn runtime_dependencies_are_resolved_recursively_and_system_libraries_are_ignored() {
    let temp = TestDir::new();
    let output = temp.0.join("output");
    let dependency = temp.0.join("dependency");
    let runtime = temp.0.join("runtime");
    let system = temp.0.join("system");
    for directory in [&output, &dependency, &runtime, &system] {
      fs::create_dir_all(directory).unwrap();
    }

    let root = output.join("libroot.so");
    let third_party = dependency.join("libthird.so");
    let nested = dependency.join("libnested.so");
    let cpp_runtime = runtime.join("libc++_shared.so");
    let libc = system.join("libc.so");
    for file in [&root, &third_party, &nested, &cpp_runtime, &libc] {
      fs::write(file, []).unwrap();
    }

    let libraries = resolve_runtime_libraries_with(
      std::slice::from_ref(&root),
      std::slice::from_ref(&dependency),
      std::slice::from_ref(&runtime),
      std::slice::from_ref(&system),
      |artifact| {
        let name = artifact.file_name().unwrap().to_string_lossy();
        Ok(
          match name.as_ref() {
            "libroot.so" => vec!["libthird.so", "libc++_shared.so", "libc.so"],
            "libthird.so" => vec!["libnested.so", "libc.so"],
            "libnested.so" | "libc++_shared.so" => Vec::new(),
            other => panic!("unexpected artifact: {other}"),
          }
          .into_iter()
          .map(str::to_owned)
          .collect(),
        )
      },
    )
    .unwrap();

    assert_eq!(
      libraries,
      vec![
        RuntimeLibrary {
          source: third_party.canonicalize().unwrap(),
          soname: "libthird.so".to_owned(),
        },
        RuntimeLibrary {
          source: cpp_runtime.canonicalize().unwrap(),
          soname: "libc++_shared.so".to_owned(),
        },
        RuntimeLibrary {
          source: nested.canonicalize().unwrap(),
          soname: "libnested.so".to_owned(),
        },
      ]
    );
  }

  #[test]
  fn unresolved_runtime_dependencies_fail_the_build() {
    let temp = TestDir::new();
    let root = temp.0.join("libroot.so");
    fs::write(&root, []).unwrap();

    let error = resolve_runtime_libraries_with(std::slice::from_ref(&root), &[], &[], &[], |_| {
      Ok(vec!["libmissing.so".to_owned()])
    })
    .unwrap_err();

    assert!(error.to_string().contains("libmissing.so"));
    assert!(error.to_string().contains("libroot.so"));
  }
}
