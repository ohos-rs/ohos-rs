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

  let args = positional("CARGO_ARGS")
    .help("Provide the ohpm environment for executing other cargo commands.")
    .many();

  let cargo_parser = construct!(crate::CargoArgs { arch, args });
  construct!(crate::Options::Cargo(cargo_parser))
}
