use bpaf::{construct, long, positional, Parser};

use crate::util::Arch;

pub fn cli_build() -> impl Parser<crate::Options> {
  let dist = long("dist")
    .argument::<String>("DIST")
    .help("The path of the final build product is set to `dist` by default.")
    .fallback(String::from("dist"));

  let release = long("release")
    .help("Build with release mode.")
    .switch()
    .fallback(false);

  let arch = long("arch")
      .short('a')
      .help("The target build products support arm64/aarch, arm/arm32, and x86_64/x64 architectures, with all builds enabled by default.")
      .argument::<Arch>("ARCH")
      .some("Please provide at least one architecture")
      .optional()
      .fallback(Some([Arch::ARM64,Arch::ARM32,Arch::X86_64].to_vec()));

  let copy_static = long("static").help("Copy the static link library to the final output directory, will be set to false by default.").switch().fallback(false);

  let cargo_args = positional("CARGO_ARGS")
    .help("The custom parameters for cargo build in the current project.")
    .strict()
    .many()
    .optional();

  let init_parser = construct!(crate::BuildArgs {
    dist,
    arch,
    release,
    copy_static,
    cargo_args,
  });
  construct!(crate::Options::Build(init_parser))
}
