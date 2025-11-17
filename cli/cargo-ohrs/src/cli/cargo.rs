use bpaf::{construct, long, positional, Parser};

use crate::util::Arch;

pub fn cli_cargo() -> impl Parser<crate::Options> {
  let arch = long("arch")
  .short('a')
  .help("Support building arm64/aarch, arm/arm32, and x86_64/x64 architecture targets, by default only build arm64.")
  .argument::<Arch>("ARCH")
  .some("Please provide at least one architecture")
  .optional()
  .fallback(Some([Arch::ARM64].to_vec()));

  let disable_target = long("disable-target")
    .help("Disable the default target argument and cmd only run once.")
    .switch()
    .fallback(false);

  let args = positional("CARGO_ARGS")
    .help("Provide the ohpm environment for executing other cargo commands.")
    .many();

  let bisheng = long("bisheng")
    .help("Use bisheng to run the project, will be set to false by default.")
    .flag(true, false);

  let cargo_parser = construct!(crate::CargoArgs {
    arch,
    bisheng,
    disable_target,
    args
  });
  construct!(crate::Options::Cargo(cargo_parser))
}
