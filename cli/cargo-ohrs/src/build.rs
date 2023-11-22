use std::process::Command;

pub fn build() {
  let targets = [
    "aarch64-unknown-linux-ohos",
    "armv7-unknown-linux-ohos",
    "x86_64-unknown-linux-ohos",
  ];

  for target in &targets {
    let output = Command::new("cargo")
      .arg("+nightly")
      .arg("build")
      .arg("--target")
      .arg(target)
      .arg("-Z")
      .arg("build-std")
      .arg("--release")
      .output()
      .expect("Failed to execute command");

    if output.status.success() {
      println!("Build for target {} succeeded", target);
    } else {
      eprintln!("Build for target {} failed", target);
      eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
      eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
  }
}
