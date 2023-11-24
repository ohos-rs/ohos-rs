use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

macro_rules! create_dist_dir {
  ($dir: expr, $target: expr) => {{
    if !$target.join($dir).as_path().exists() {
      let _ = std::fs::create_dir($target.join($dir))
        .expect(format!("Can't create {} dir.", $dir).as_str());
    }
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
  let targets = [
    ("arm64-v8a", "aarch64-unknown-linux-ohos", "arm64"),
    ("armeabi-v7a", "armv7-unknown-linux-ohos", "arm"),
    ("x86_64", "x86_64-unknown-linux-ohos", "x86_64"),
  ];
  let mut mode_arg = "";
  let mut mode = "debug";
  if release {
    mode_arg = "--release";
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

  for (target_dir, target, platform) in &targets {
    let all_args = [
      "+nightly",
      "build",
      "--target",
      target,
      "-Z",
      "build-std",
      mode_arg,
    ];

    let args: Vec<_> = all_args.iter().filter(|v| !v.is_empty()).collect();

    let mut child = Command::new("cargo")
      .args(args)
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

    let source = pwd.join("target").join(target).join(mode);

    let files: Vec<PathBuf> = fs::read_dir(source)
      .expect("Failed to read directory")
      .filter_map(Result::ok)
      .map(|entry| entry.path())
      .filter(|path| path.is_file() && path.extension().map_or(false, |e| e == "so"))
      .collect();

    let mut compact_dir = "";

    if !compact {
      compact_dir = target_dir;
    }

    let bin_dir = &bin_target_dir.join(compact_dir);

    for file in files {
      let dist: PathBuf;
      if !compact {
        dist = bin_dir.join(file.file_name().unwrap());
      } else {
        let file_name = file.file_stem().unwrap().to_str().unwrap();
        dist = bin_dir.join(format!("{}_{}.so", file_name, platform));
      }
      move_file!(file, dist);
    }
  }
}
