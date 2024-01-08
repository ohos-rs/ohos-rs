use clap::Parser;

mod arg;
mod build;
mod doctor;
mod init;
mod marco;

fn main() {
  let cli = arg::OhrsCli::parse();
  match cli.command {
    arg::Commands::Init(init_arg) => {
      init::init(init_arg);
    }
    arg::Commands::Build(build_arg) => {
      {
        let mut arg = build::BUILD_ARGS.write().unwrap();
        *arg = build_arg;
      }
      build::build();
    }
    arg::Commands::Doctor => {
      doctor::doctor();
    }
  }
}
