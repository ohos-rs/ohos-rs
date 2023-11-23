use std::io::{BufRead, BufReader};
use std::process::{exit, Command, Stdio};

macro_rules! create_dist_dir {
  ($dir: expr, $target: expr) => {{
    let _ = std::fs::create_dir($target.join($dir))
      .expect(format!("Can't create {} dir.", $dir).as_str());
  }};
}

macro_rules! move_file {
  ($source: expr,$dist: expr) => {{
    let mut options = fs_extra::file::CopyOptions::new();
    // if exist will overwrite
    options = options.overwrite(true);
    fs_extra::file::move_file($source, $dist, &options).unwrap();
  }};
}

pub fn build(dir: String, compact: bool, release: bool) {

  let name = "";

  let targets = [
    ("arm64-v8a", "aarch64-unknown-linux-ohos"),
    ("armeabi-v7a", "armv7-unknown-linux-ohos"),
    ("x86_64", "x86_64-unknown-linux-ohos"),
  ];
  let mut mode = "debug";
  if release {
    mode = "release";
  }

  let pwd = std::env::current_dir().unwrap();

  create_dist_dir!(dir.as_str(), &pwd);

  let bin_target_dir = pwd.join(dir.as_str());

  if !compact {
    create_dist_dir!("arm64-v8a", &bin_target_dir);
    create_dist_dir!("armeabi-v7a", &bin_target_dir);
    create_dist_dir!("x86_64", &bin_target_dir);
  }

  for (target_dir, target) in &targets {
    let mut child = Command::new("cargo")
      .arg("+nightly")
      .arg("build")
      .arg("--target")
      .arg(target)
      .arg("-Z")
      .arg("build-std")
      .arg(format!("--{}", mode))
      .stdout(Stdio::piped())
      .spawn()
      .expect("Failed to execute command");

    if let Some(ref mut stdout) = child.stdout {
      let reader = BufReader::new(stdout);

      for line in reader.lines() {
        let line = line.expect("Failed to read line");
        println!("{}", line);
      }
    }

    let output = child.wait_with_output().expect("Failed to wait on child");

    if output.status.success() {
      println!("Build for target {} succeeded", target);
    } else {
      eprintln!("Build for target {} failed", target);
      eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
      eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
      exit(-1);
    }

    let source = pwd
      .join("target")
      .join("aarch64-unknown-linux-ohos")
      .join(target)
      .join(format!("lib${}.so", name));

    let mut compact_dir = "";

    if !compact {
      compact_dir = target_dir;
    }

    let dist = &bin_target_dir.join(compact_dir);
    
    move_file!(source, dist);
  }
}
