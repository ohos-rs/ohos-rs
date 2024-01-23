use crate::build::{Architecture, Context};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{exit, Command, Stdio};

pub fn build(ctx: &mut Context, arch: &Architecture) {
  let mut args = ctx.init_args.clone();
  args.extend(["--target", &arch.target]);
  let ndk = env::var("OHOS_NDK_HOME").expect("OHOS_NDK_HOME not set");

  let ohos_c_cflags = HashMap::from([
        ("arm64", format!("-target aarch64-linux-ohos --sysroot={}/native/sysroot -D__MUSL__", ndk)),
        ("arm", format!("-target arm-linux-ohos --sysroot={}/native/sysroot -D__MUSL__ -march=armv7-a -mfloat-abi=softfp -mtune=generic-armv7-a -mthumb", ndk)),
        ("x86_64", format!("-target x86_64-linux-ohos --sysroot={}/native/sysroot -D__MUSL__", ndk)),
    ]);

  let linker = &format!(
    "CARGO_TARGET_{}_LINKER",
    &arch.target.to_case(Case::UpperSnake)
  );

  let build_env_map = HashMap::from([
    ("CC", format!("{}/native/llvm/bin/clang", ndk)),
    ("CXX", format!("{}/native/llvm/bin/clang++", ndk)),
    ("AR", format!("{}/native/llvm/bin/llvm-ar", ndk)),
    ("RANLIB", format!("{}/native/llvm/bin/llvm-ranlib", ndk)),
    (linker, format!("{}/native/llvm/bin/clang", ndk)),
    (
      "CFLAGS",
      ohos_c_cflags.get(arch.platform).unwrap().to_string(),
    ),
  ]);

  println!("{:?}",build_env_map);

  let mut child = Command::new("cargo")
    .args(args)
    .envs(build_env_map)
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
