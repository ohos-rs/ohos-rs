use crate::build::{Architecture, Context};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{exit, Command, Stdio};
use std::sync::{Arc, RwLock};

static TARGET: Lazy<HashMap<&str, (&str, &str)>> = Lazy::new(|| {
  HashMap::from([
    (
      "arm64-v8a",
      ("aarch64-linux-ohos", "AARCH64_UNKNOWN_LINUX_OHOS"),
    ),
    (
      "armeabi-v7a",
      ("arm-linux-ohos", "ARMV7_UNKNOWN_LINUX_OHOS"),
    ),
    ("x86_64", ("x86_64-linux-ohos", "X86_64_UNKNOWN_LINUX_OHOS")),
  ])
});

pub fn build(c: Arc<RwLock<Context>>, arch: &Architecture) {
  let ctx = c.read().unwrap();
  let t = TARGET.get(arch.arch).unwrap();
  let linker_name = format!("CARGO_TARGET_{}_LINKER", t.1);
  let ran_path = format!("{}/native/llvm/bin/llvm-ranlib", &ctx.ndk);
  let ar_path = format!("{}/native/llvm/bin/llvm-ar", &ctx.ndk);
  let cc_path = format!("{}/native/llvm/bin/clang", &ctx.ndk);
  let cxx_path = format!("{}/native/llvm/bin/clang++", &ctx.ndk);
  let mut rustflags = format!(
    "-Clink-args=-target {} --sysroot={}/native/sysroot -D__MUSL__",
    t.0, &ctx.ndk
  );

  if arch.arch == "armeabi-v7a" {
    rustflags = format!(
      "{} -march=armv7-a -mfloat-abi=softfp -mtune=generic-armv7-a -mthumb",
      rustflags
    );
  }

  let prepare_env = HashMap::from([
    (linker_name.as_str(), &cc_path),
    ("CC", &cc_path),
    ("CXX", &cxx_path),
    ("RANLIB", &ran_path),
    ("AR", &ar_path),
    ("CARGO_ENCODED_RUSTFLAGS", &rustflags),
  ]);

  (&prepare_env).iter().for_each(|(k, v)| {
    env::set_var(k, v);
  });

  let mut args = ctx.init_args.clone();
  args.extend(["--target", &arch.target]);

  let mut child = Command::new("cargo")
    .args(args)
    .envs(&prepare_env)
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to execute command");

  if let Some(ref mut stdout) = child.stdout {
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
      let line = line.expect("Failed to read line");
      println!("{}", line);
    }
    let output = child.wait_with_output().expect("Failed to wait on child");

    if output.status.success() {
      println!("Build for target {} succeeded", &arch.target);
    } else {
      eprintln!("Build for target {} failed", &arch.target);
      eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
      eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
      exit(-1);
    }
  }
}
