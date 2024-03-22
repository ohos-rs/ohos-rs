use crate::build::Context;
use std::{
  process::Command,
  sync::{Arc, RwLock},
  thread::{self, JoinHandle},
};

pub fn strip(c: Arc<RwLock<Context>>, threads: &mut Vec<JoinHandle<()>>) {
  let args = super::BUILD_ARGS.read().unwrap();
  let ctx = c.read().unwrap();
  let strip = args.strip.unwrap_or(args.release);
  if strip {
    let llvm_strip = Arc::new(format!("{}/native/llvm/bin/llvm-strip", ctx.ndk));
    ctx.dist_files.iter().for_each(|p| {
      let a = p.clone();
      let command = llvm_strip.clone();
      threads.push(thread::spawn(move || {
        Command::new(command.as_str())
          .arg(a.to_str().unwrap())
          .spawn()
          .unwrap();
      }))
    });
  }
}
