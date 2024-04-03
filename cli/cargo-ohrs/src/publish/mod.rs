use crate::arg::PublishArg;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{env, thread};

pub fn publish(arg: PublishArg) {
  let pkg = arg.package.unwrap_or(String::from("package.har"));
  let mut pwd = env::current_dir().unwrap();
  let default_path = PathBuf::from(&pkg);
  if default_path.is_relative() {
    pwd = pwd.join(&pkg)
  } else {
    pwd = PathBuf::from(&pkg);
  }

  let mut child = Command::new("ohpm")
    .arg("publish")
    .arg(pwd)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to spawn ohpm command");

  let mut stdin = child.stdin.take().expect("failed to get stdin");
  let mut stdout = child.stdout.take().expect("failed to get stdout");

  let handle = thread::spawn(move || {
    let mut buffer = [0; 1024];
    loop {
      match stdout.read(&mut buffer) {
        Ok(n) if n > 0 => {
          // 处理读取到的数据
          let output = String::from_utf8_lossy(&buffer[..n]);
          if output.contains("key_path error.") {
            continue;
          }
          print!("{}", output.is_empty());
          print!("{}", output);
          if output.contains("private key:") {
            print!("test");
            // 检测到等待输入的提示
            break;
          }
        }
        Ok(_) => {
          println!("等待输入");
          // 没有数据读取，短暂休眠后继续尝试
          thread::sleep(Duration::from_millis(100));
        }
        Err(e) => {
          eprintln!("读取错误: {}", e);
          break;
        }
      }
    }
  });

  handle.join().unwrap();
}
