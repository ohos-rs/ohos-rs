use crate::build::{Architecture, Context};
use cargo_metadata::Message;
use std::io::{BufRead, BufReader};
use std::process::{exit, Command, Stdio};

pub fn build(ctx: &mut Context, arch: &Architecture) {
  let mut args = ctx.init_args.clone();
  args.extend(["--target", &arch.target]);

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
  // 通过构建命令获取metadata 只能用单线程多线程会导致block
  let mut meta_args = ctx.init_args.clone();
  meta_args.extend([
    "--target",
    &arch.target,
    "--message-format=json-render-diagnostics",
  ]);

  let mut child = Command::new("cargo")
    .args(meta_args)
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to execute command");

  let reader = BufReader::new(child.stdout.take().unwrap());
  for message in cargo_metadata::Message::parse_stream(reader) {
    match message.unwrap() {
      Message::CompilerArtifact(artifact) => {
        let cargo_file = (&ctx.pwd).join("Cargo.toml");
        if artifact.manifest_path.eq(cargo_file.to_str().unwrap()) {
          ctx.artifact = Some(artifact);
        }
      }
      _ => (), // Unknown message
    }
  }

  let _ = child.wait().expect("Couldn't get cargo's exit status");
}
