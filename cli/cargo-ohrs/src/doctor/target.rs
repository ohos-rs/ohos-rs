use std::process::Command;

use anyhow::{anyhow, Ok};

pub fn resolve_targets() -> anyhow::Result<Vec<String>> {
  let output = Command::new("rustup")
    .args(&["target", "list", "--installed"])
    .output()
    .map_err(|_| anyhow!("Can't run rustup command, please check you already have install it."))?;

  let mut targets = Vec::new();

  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().for_each(|line| {
      targets.push(String::from(line));
    });
  } else {
    return Err(anyhow!(
      "Try to get installed targets list failed: {}",
      String::from_utf8_lossy(&output.stderr)
    ));
  }
  Ok(targets)
}

pub fn resolve_rust_version() -> anyhow::Result<String> {
  let version_output = Command::new("rustc")
    .arg("--version")
    .output()
    .map_err(|_| {
      anyhow!("Can't run rustc command, please check you already have install rustup or rust.")
    })?;

  if version_output.status.success() {
    let version = String::from_utf8_lossy(&version_output.stdout);
    let version_parts: Vec<&str> = version.split_whitespace().collect();
    if version_parts.len() > 1 {
      Ok(String::from(version_parts[1]))
    } else {
      Ok(String::from("0.0.0"))
    }
  } else {
    Err(anyhow!(
      "Try to get rust version failed: {}",
      String::from_utf8_lossy(&version_output.stderr)
    ))
  }
}
