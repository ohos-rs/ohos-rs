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
  Init { 
    /// Project name,which will be created.
    name: String 
  },
  /// Project construction
  Build {
    /// dist target dir default is dist
    #[arg(long,short,default_value_t = String::from("dist"))]
    dir: String,
    /// dist file is compact default is false
    /// arm64-v8a/armeabi-v7a/x86_64
    #[arg(long,short,default_value_t = false)]
    compact: bool,

    /// build target with release mode default is false
    #[arg(long,default_value_t = false)]
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
      let dist_dir = dir.clone();
      let compact_flag = compact.clone();
      let release_flag = release.clone();
      build::build(dist_dir, compact_flag, release_flag);
    }
    Commands::Doctor => {
      doctor::doctor();
    }
  }
}
