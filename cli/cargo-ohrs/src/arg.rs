use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
  author,
  version,
  about,
  long_about = "The ohos-rs scaffold tool is used for project initialization, project construction, and environment checks, etc."
)]
pub struct OhrsCli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
  /// Project initialization
  Init(InitArg),
  /// Project construction
  Build(BuildArg),
  /// Check environments
  Doctor,
}

#[derive(Args)]
pub struct InitArg {
  /// Project name,which will be created.
  pub name: String,
}

#[derive(Args, Default)]
pub struct BuildArg {
  /// dist target dir default is dist
  #[arg(long, short, default_value_t = String::from("dist"))]
  pub dir: String,
  #[arg(
    long,
    short,
    default_value_t = false,
    help = "dist file is compact default is false"
  )]
  pub compact: bool,

  /// build target with release mode default is false
  #[arg(long, default_value_t = false)]
  pub release: bool,
}
