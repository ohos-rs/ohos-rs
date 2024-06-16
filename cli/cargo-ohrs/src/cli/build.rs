use bpaf::{construct, long, positional, Parser};
use std::str::FromStr;

#[derive(Debug, Clone, Copy,PartialEq)]
pub enum Arch {
  ARM64,
  ARM32,
  X86_64,
}

impl Arch {
  pub fn to_arch(self) -> &'static str {
    match self {
      Arch::ARM64 => "aarch",
      Arch::ARM32 => "arm",
      Arch::X86_64 => "x86_64",
    }
  }
}

impl FromStr for Arch {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, String>
  where
    Self: Sized,
  {
    let ret = s.to_lowercase();
    match ret.as_ref() {
      "aarch" | "arm64" => Ok(Arch::ARM64),
      "arm" | "arm32" => Ok(Arch::ARM32),
      "x86_64" | "x64" => Ok(Arch::X86_64),
      _ => Err("Only supports aarch/arm64, arm/arm32, and x86_64/x64 architectures.".to_string()),
    }
  }
}

pub fn cli_build() -> impl Parser<crate::Options> {
  let dist = long("dist")
    .argument::<String>("DIST")
    .help("The path of the final build product is set to `dist` by default.")
    .fallback(String::from("dist"));

  let arch = long("arch")
      .short('a')
      .help("The target build products support arm64/aarch, arm/arm32, and x86_64/x64 architectures, with all builds enabled by default.")
      .argument::<Arch>("ARCH")
      .many()
      .optional();

  let cargo_args = positional("CARGO_ARGS")
    .help("The custom parameters for cargo build in the current project.")
    .strict()
    .many()
    .optional();

  let init_parser = construct!(crate::BuildArgs {
    dist,
    arch,
    cargo_args
  });
  construct!(crate::Options::Build(init_parser))
}
