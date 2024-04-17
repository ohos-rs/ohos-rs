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
  /// TODO: need to implement
  Doctor,
  /// Publish ohpm's package
  Publish(PublishArg),
  /// Generate har package
  Artifact(ArtifactArg),
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

  /// build target with release mode default is false
  #[arg(short = 'r', long, default_value_t = false)]
  #[arg(
    short = 'd',
    long,
    default_value_t = String::from("dist"),
    help="Target's file will be copied to this folder. "
  )]
  pub dist: String,

  #[arg(
    long,
    short = 'c',
    default_value_t = false,
    help = "The product file use compact mode"
  )]
  pub compact: bool,

  #[arg(
    short = 'r',
    long,
    default_value_t = false,
    help = "Whether to build in release mode, the default is false."
  )]
  pub release: bool,

  #[arg(
    short = 's',
    long,
    num_args(0..=1),
    require_equals(true),
    default_missing_value("true"),
    help = "Use llvm-strip to reduce the binary size, which is true by default in release mode.",
    action = clap::ArgAction::Set
  )]
  pub strip: Option<bool>,
}

#[derive(Args, Default)]
pub struct PublishArg {
  #[arg(long, help = "ohpm's token, will use it to publish.")]
  pub token: String,

  #[arg(long, help = "har package's path,default is $PWD/package.har")]
  pub package: Option<String>,
}

#[derive(Args, Default)]
pub struct ArtifactArg {
  #[arg(
    long,
    default_value_t = String::from("package"),
    help = "The package name of the generated har, default is package"
  )]
  pub name: String,

  #[arg(long,
    short = 'd',
    default_value_t = String::from("dist"),
    help = "This folder will copy to package/libs"
  )]
  pub dist: String,
}
