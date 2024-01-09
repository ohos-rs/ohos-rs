use crate::build::{Architecture, Context};
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
}
