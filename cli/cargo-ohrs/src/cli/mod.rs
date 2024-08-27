mod artifact;
mod build;
mod cargo;
mod init;
mod publish;

use bpaf::{construct, pure, Doc, OptionParser, Parser};
use owo_colors::colors::CustomColor;
use owo_colors::OwoColorize;

use artifact::cli_artifact;
use build::cli_build;
use cargo::cli_cargo;
use init::cli_init;
use publish::cli_publish;

pub fn cli_run() -> OptionParser<crate::Options> {
  let init = cli_init()
    .to_options()
    .command("init")
    .help("Project initialization");
  let build = cli_build()
    .to_options()
    .command("build")
    .help("Project build");
  let artifact = cli_artifact()
    .to_options()
    .command("artifact")
    .help("Generate ohpm .har file");
  let publish = cli_publish()
    .to_options()
    .command("publish")
    .help("Publish ohpm package, but not implement yet.");
  let cargo = cli_cargo()
    .to_options()
    .command("cargo")
    .help("Used to execute any cargo command and ensure it is under the ohpm environment.");

  let doctor = pure(crate::Options::Doctor)
    .to_options()
    .command("doctor")
    .help("Verify if the development environment is fully set up.");

  construct!([init, build, artifact, publish, cargo, doctor]).to_options()
}

pub struct Info();

static LOGO: &str = r#"
        _                              
   ___ | |__   ___  ___       _ __ ___ 
  / _ \| '_ \ / _ \/ __|_____| '__/ __|
 | (_) | | | | (_) \__ \_____| |  \__ \
  \___/|_| |_|\___/|___/     |_|  |___/                                       
"#;

impl From<Info> for Doc {
  fn from(_value: Info) -> Self {
    let mut doc = Self::default();
    doc.text(
      LOGO
        .fg::<CustomColor<248, 112, 51>>()
        .bold()
        .to_string()
        .as_str(),
    );
    doc.text(
      "\n \n This command is used for building native modules of Harmony in the ohos-rs project."
        .blue()
        .to_string()
        .as_str(),
    );
    doc.text("\n It provides a range of capabilities including project initialization, building, CI/CD, etc.".blue().to_string().as_str());
    doc
  }
}
