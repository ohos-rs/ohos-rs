use owo_colors::OwoColorize;
use util::Arch;

use crate::cli::cli_run;

mod artifact;
mod build;
mod cargo;
mod cli;
mod doctor;
mod init;
mod publish;
mod util;

#[derive(Debug, Clone)]
pub(crate) struct InitArgs {
  package_name: Option<String>,
  name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildArgs {
  dist: String,
  arch: Option<Vec<Arch>>,
  release: bool,
  cargo_args: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ArtifactArgs {
  dist: String,
  name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CargoArgs {
  arch: Option<Vec<Arch>>,
  args: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum Options {
  Init(InitArgs),
  Build(BuildArgs),
  Artifact(ArtifactArgs),
  Cargo(CargoArgs),
  #[allow(dead_code)]
  Publish,
  #[allow(dead_code)]
  Doctor,
}

fn main() {
  let parser = cli_run()
    .descr(cli::Info())
    .version(env!("CARGO_PKG_VERSION"));

  let ret = parser.fallback_to_usage().run();

  let run_ret = match ret {
    Options::Init(args) => init::init(args),
    Options::Build(args) => build::build(args),
    Options::Artifact(args) => artifact::artifact(args),
    Options::Cargo(args) => cargo::cargo(args),
    Options::Doctor => doctor::doctor(),
    Options::Publish => publish::publish(),
  };
  if let Err(e) = run_ret {
    println!("{:?}", e.red());
  }
}
