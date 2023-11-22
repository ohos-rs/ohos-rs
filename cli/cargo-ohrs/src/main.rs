use std::process::exit;

use clap::Parser;

mod build;
mod doctor;
mod init;
mod tmps;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum OhrsCli {
  #[command(about = "A cli tool for ohos-rs")]
  OHRS(Ohrs),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
struct Ohrs {
  /// build target
  #[arg(short, long)]
  build: bool,

  /// init project
  #[arg(short, long)]
  init: Option<String>,

  /// validate env
  #[arg(short, long)]
  doctor: bool,
}

fn main() {
  let arg = OhrsCli::parse();
  match arg {
    OhrsCli::OHRS(args) => {
      let build_flag = args.build;
      let init_flag = args.init.is_some();
      let doctor_flag = args.doctor;
      let mut count = 0;
      if build_flag {
        count = count + 1;
        print!("log1");
      }
      if init_flag {
        count = count + 1;
      }
      if doctor_flag {
        count = count + 1;
      }
      if count == 0 {
        println!("Must provide one of --build,--init,--doctor");
        exit(-1);
      }
      if count > 1 {
        println!("Received too many arguments, just provide one of --build,--init,--doctor");
        exit(-1)
      }
      if build_flag {
        build::build();
        return;
      }
      if let Some(name) = args.init {
        init::init(name);
        return;
      }
      if doctor_flag {
        doctor::doctor();
      }
    }
  }
}
