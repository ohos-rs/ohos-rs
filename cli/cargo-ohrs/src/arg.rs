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
  /// Initialize project
  Init(InitArg),
  /// Build project
  Build(BuildArg),
  /// Check environments
  Doctor,
  /// Publish ohpm's package
  Publish(PublishArg)
}

#[derive(Args)]
pub struct InitArg {
  /// Project's name.Folder will be created, if the folder is already existed and will fail.
  pub name: String,
  #[arg(
    default_value_t = false,
    short = 'p',
    long,
    help = "init with ohpm package"
  )]
  pub package: bool,
  #[arg(num_args(0..=1), requires("package"), help = "ohpm package's name.if not set,will use project's name")]
  pub package_name: Option<String>,
}

#[derive(Args, Default)]
pub struct BuildArg {
  #[arg(long,
    short = 'd',
    default_value_t = String::from("dist"),
    help = "Target's file will be copied to this folder.")]
  pub dist: String,

  #[arg(
    long,
    short = 'c',
    default_value_t = false,
    help = "Target's file is compact mode"
  )]
  pub compact: bool,

  /// build target with release mode default is false
  #[arg(short = 'r', long, default_value_t = false)]
  pub release: bool,
}

#[derive(Args, Default)]
pub struct PublishArg {
  #[arg(long, help = "ohpm's token, will use it to publish.")]
  pub token: String,
}
