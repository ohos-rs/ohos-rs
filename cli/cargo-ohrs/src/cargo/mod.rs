use anyhow::Error;
use std::env;

use crate::util::Arch;

mod run;

pub fn cargo(args: crate::CargoArgs) -> anyhow::Result<()> {
  let ndk = env::var("OHOS_NDK_HOME").map_err(|_| {
    Error::msg(
      "Failed to get the OHOS_NDK_HOME environment variable, please make sure you have set it.",
    )
  })?;
  let (command, rest_args) = args.args.split_at(1);
  let mut target_arch = args.arch.unwrap_or(vec![Arch::ARM64]);

  // if disable-target is true, just run once.
  if args.disable_target {
    target_arch = vec![Arch::ARM64]
  }

  target_arch
    .iter()
    .map(|arch| {
      let mut all_args = Vec::new();

      all_args.extend(command);

      let t = String::from("--target");
      let rt = String::from(arch.rust_target());

      if !args.disable_target {
        all_args.extend([&t, &rt]);
      }

      all_args.extend(rest_args);

      run::run(arch, ndk.clone(), all_args)?;
      Ok(())
    })
    .collect::<anyhow::Result<Vec<_>>>()?;

  Ok(())
}
