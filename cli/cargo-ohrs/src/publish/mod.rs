use crate::arg::PublishArg;
use std::env;
use std::io::BufReader;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::sleep;

pub fn publish(arg: PublishArg) {
  let pkg = arg.package.unwrap_or(String::from("package.har"));
  let mut pwd = env::current_dir().unwrap();
  let default_path = PathBuf::from(&pkg);
  if default_path.is_relative() {
    pwd = pwd.join(&pkg)
  } else {
    pwd = PathBuf::from(&pkg);
  }

  println!("{:?}",pwd);
  let mut child = Command::new("ohpm")
    .arg("publish")
    .arg(pwd)
    .stdin(Stdio::piped())
    .spawn()
    .expect("Failed to spawn ohpm command");


  // 向命令的标准输入写入私钥
  if let Some(ref mut stdin) = child.stdin.take() {
    println!("need to input");
    // let t = std::time::Duration::new(10000,0);
    // sleep(t);
    stdin
      .write_all(b"Ranger123.\r\n")
      .expect("Failed to write to stdin");
  }


  let output = child.wait_with_output().unwrap();
  println!("Output: {}", String::from_utf8_lossy(&output.stdout));

}
