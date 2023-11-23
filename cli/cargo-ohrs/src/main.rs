use clap::{Parser, Subcommand};

mod build;
mod doctor;
mod init;
mod tmps;

#[derive(Parser)]
#[command(
  author,
  version,
  about,
  long_about = "The ohos-rs scaffold tool is used for project initialization, project construction, and environment checks, etc."
)]
struct OhrsCli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Project initialization
  Init { name: String },
  /// Project construction
  Build {
    /// dist target dir default is dist
    dir: Option<String>,
    /// dist file is compact default is false
    /// arm64-v8a/armeabi-v7a/x86_64
    #[arg(default_value_t = false)]
    compact: bool,

    /// build target with release mode default is false
    #[arg(default_value_t = false)]
    release: bool,
  },
  /// Check environments
  Doctor,
}

fn main() {
  let cli = OhrsCli::parse();
  match &cli.command {
    Commands::Init { name } => {
      init::init(name.clone());
    }
    Commands::Build {
      dir,
      compact,
      release,
    } => {
      let dist_dir = dir.clone().map_or(String::from("dist"), |v| v);
      build::build(dist_dir, compact.clone(), release.clone());
    }
    Commands::Doctor => {}
  }
}
