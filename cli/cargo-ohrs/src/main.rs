use clap::{Parser, Subcommand};

mod build;
mod doctor;
mod init;
mod tmps;

#[derive(Parser)]
#[command(author,version,about = "A cli tool for ohos-rs to init,build,doctor & etc.",long_about = None)]
struct Cli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  #[command(about = "run init command")]
  Init { name: String },
  #[command(about = "run build command")]
  Build,
  #[command(about = "run doctor command")]
  Doctor,
}

fn main() {
  let arg = Cli::parse();
  match arg.command {
    Commands::Build => {
      build::build();
    }
    Commands::Init { name } => {
      init::init(name);
    }
    Commands::Doctor => {
      doctor::doctor();
    }
  }
}
