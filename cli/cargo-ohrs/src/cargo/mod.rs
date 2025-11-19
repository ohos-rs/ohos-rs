use crate::util::Arch;
use anyhow::Error;
use std::{env, str::FromStr};

mod run;

pub fn cargo(args: crate::CargoArgs) -> anyhow::Result<()> {
  let ohos_ndk = env::var("OHOS_NDK_HOME").map_err(|_| {
    Error::msg(
      "Failed to get the OHOS_NDK_HOME environment variable, please make sure you have set it.",
    )
  })?;
  if args.args.len() < 1 {
    return Err(Error::msg(
      "You don't provide any command for current command.",
    ));
  }

  let (command, rest_args) = args.args.split_at(1);
  let mut target_arch = args.arch.unwrap_or(vec![Arch::ARM64]);
  let mut target_arg = None;

  let mut iter = rest_args.iter().peekable();
  while let Some(arg) = iter.next() {
    if arg == "--target" {
      if let Some(&next_arg) = iter.peek() {
        target_arg = Some(next_arg.to_string());
        break;
      }
    }
  }

  // if disable-target is true, just run once.
  if args.disable_target {
    if let Some(t) = target_arg {
      let all_targets = [
        Arch::ARM32.rust_target(),
        Arch::ARM64.rust_target(),
        Arch::X86_64.rust_target(),
      ];
      let ret = all_targets.iter().find(|&&x| x == t.as_str());
      if let Some(r) = ret {
        let arch = Arch::from_str(r).map_err(|e| Error::msg(e))?;
        target_arch = vec![arch];
      } else {
        return Err(Error::msg("Only support ohos target"));
      }
    } else {
      return Err(Error::msg(
        "You don't provide any target for current command.",
      ));
    }
  }

  target_arch
    .iter()
    .map(|arch| {
      let mut all_args = match arch.to_arch() {
        "loongarch64" => vec!["+nightly"],
        _ => Vec::new(),
      };

      all_args.extend([command[0].as_str()]);

      if !args.disable_target {
        all_args.extend(["--target", arch.rust_target()]);
      }

      if arch.to_arch() == "loongarch64" {
        all_args.extend(["-Z", "build-std"]);
      }

      all_args.extend(rest_args.iter().map(|s| s.as_str()));

      run::run(arch, ohos_ndk.clone(), all_args, args.bisheng)?;
      Ok(())
    })
    .collect::<anyhow::Result<Vec<_>>>()?;

  Ok(())
}
