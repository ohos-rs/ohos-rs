use crate::cli::{Arch, cli_run};
use bpaf::Parser;

mod artifact;
mod build;
mod cli;
mod doctor;
mod init;
mod marco;
mod publish;

#[derive(Debug, Clone)]
pub(crate) struct InitArgs {
  package_name: Option<String>,
  name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildArgs {
  dist: String,
  arch: Option<Vec<Arch>>,
  cargo_args: Option<Vec<String>>
}

#[derive(Debug, Clone)]
pub(crate) struct ArtifactArgs {
  dist: String,
  name: String,
}

#[derive(Debug, Clone)]
pub(crate) enum Options {
  Init(InitArgs),
  Build(BuildArgs),
  Artifact(ArtifactArgs),
}

fn main() {
  let parser = cli_run().descr(cli::Info());

  let ret = parser.fallback_to_usage().run();

  println!("{:?}",ret);

  // match ret {
  //   Options::Init(args) => init::init(args),
  //   Options::Build(args) => build::build(args),
  //   Options::Artifact(args) => artifact::artifact(args),
  // }
}
