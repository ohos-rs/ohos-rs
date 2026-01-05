use crate::util::Arch;
use anyhow::Error;
use cargo_metadata::{MetadataCommand, Package};
use std::{env, str::FromStr};

mod run;

fn get_workspace_packages() -> anyhow::Result<Vec<Package>> {
  let pwd = env::current_dir()?;
  let cargo_file = pwd.join("./Cargo.toml");

  if cargo_file.try_exists().is_err() {
    return Ok(vec![]);
  }

  let metadata = MetadataCommand::new()
    .no_deps()
    .manifest_path(&cargo_file)
    .exec()?;

  let is_workspace = !metadata.workspace_members.is_empty();

  if is_workspace {
    let packages: Vec<Package> = metadata
      .workspace_members
      .iter()
      .filter_map(|member_id| {
        metadata
          .packages
          .iter()
          .find(|p| &p.id == member_id)
          .cloned()
      })
      .collect();
    Ok(packages)
  } else {
    Ok(vec![])
  }
}

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

  // 解析 package 参数（从 -p 参数或 args 中的 -p）
  let package_filter = args.package.clone().or_else(|| {
    rest_args
      .iter()
      .position(|arg| arg == "-p" || arg == "--package")
      .and_then(|idx| rest_args.get(idx + 1).cloned())
  });

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

  //
  let workspace_packages = get_workspace_packages()?;
  let is_workspace = workspace_packages.len() > 1;

  // 如果是 workspace 模式，为每个包分别执行命令
  if is_workspace {
    // 如果指定了 package 参数，只处理指定的包
    let packages_to_process: Vec<&Package> = if let Some(ref pkg_name) = package_filter {
      workspace_packages
        .iter()
        .filter(|p| p.name == *pkg_name)
        .collect()
    } else {
      workspace_packages.iter().collect()
    };

    if packages_to_process.is_empty() {
      if let Some(ref pkg_name) = package_filter {
        return Err(Error::msg(format!(
          "Package '{}' not found in workspace",
          pkg_name
        )));
      }
    }

    for pkg in packages_to_process {
      println!("Running cargo command for package: {}", pkg.name);

      target_arch
        .iter()
        .map(|arch| {
          let mut all_args = match arch.to_arch() {
            "loongarch64" => vec!["+nightly"],
            _ => Vec::new(),
          };

          all_args.extend([command[0].as_str()]);
          // 如果 rest_args 中没有 -p 参数，则添加
          if !rest_args
            .iter()
            .any(|arg| arg == "-p" || arg == "--package")
          {
            all_args.extend(["-p", &pkg.name]);
          }

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
    }
  } else {
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
  }

  Ok(())
}
