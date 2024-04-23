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
  #[arg(
    short = 'd',
    long,
    default_value_t = String::from("dist"),
    help="Target's file will be copied to this folder. "
  )]
  pub dist: String,

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

  #[arg(
    long,
    num_args(0..=3),
    help = "Construct the specified architectural output. By default, all are built.",
    value_parser = validate_arch
  )]
  pub arch: Option<Vec<String>>,
}

const VALID_ARCH: [&str; 3] = ["aarch", "arm", "x64"];

fn validate_arch(value: &str) -> Result<String, String> {
  let result = VALID_ARCH.contains(&value);
  match result {
    true => Ok(value.to_string()),
    false => Err(format!(
      "{} is not supported, just support: aarch arm x64",
      value
    )),
  }
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