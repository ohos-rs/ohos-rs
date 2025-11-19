use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Arch {
  ARM64,
  ARM32,
  X86_64,
  LoongArch64,
}

impl Arch {
  pub fn to_arch(self) -> &'static str {
    match self {
      Arch::ARM64 => "arm64-v8a",
      Arch::ARM32 => "armeabi-v7a",
      Arch::X86_64 => "x86_64",
      Arch::LoongArch64 => "loongarch64",
    }
  }
  pub fn c_target(self) -> &'static str {
    match self {
      Arch::ARM64 => "aarch64-linux-ohos",
      Arch::ARM32 => "arm-linux-ohos",
      Arch::X86_64 => "x86_64-linux-ohos",
      Arch::LoongArch64 => "loongarch64-linux-ohos",
    }
  }
  pub fn rust_link_target(self) -> &'static str {
    match self {
      Arch::ARM64 => "AARCH64_UNKNOWN_LINUX_OHOS",
      Arch::ARM32 => "ARMV7_UNKNOWN_LINUX_OHOS",
      Arch::X86_64 => "X86_64_UNKNOWN_LINUX_OHOS",
      Arch::LoongArch64 => "LOONGARCH64_UNKNOWN_LINUX_OHOS",
    }
  }

  pub fn rust_target(self) -> &'static str {
    match self {
      Arch::ARM64 => "aarch64-unknown-linux-ohos",
      Arch::ARM32 => "armv7-unknown-linux-ohos",
      Arch::X86_64 => "x86_64-unknown-linux-ohos",
      Arch::LoongArch64 => "loongarch64-unknown-linux-ohos",
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
      "aarch"
      | "arm64"
      | "aarch64-linux-ohos"
      | "aarch64-unknown-linux-ohos"
      | "AARCH64_UNKNOWN_LINUX_OHOS" => Ok(Arch::ARM64),
      "arm"
      | "arm32"
      | "arm-linux-ohos"
      | "armv7-unknown-linux-ohos"
      | "ARMV7_UNKNOWN_LINUX_OHOS" => Ok(Arch::ARM32),
      "x86_64"
      | "x64"
      | "x86_64-linux-ohos"
      | "x86_64-unknown-linux-ohos"
      | "X86_64_UNKNOWN_LINUX_OHOS" => Ok(Arch::X86_64),
      "loongarch64"
      | "loongarch64-linux-ohos"
      | "loongarch64-unknown-linux-ohos"
      | "LOONGARCH64_UNKNOWN_LINUX_OHOS" => Ok(Arch::LoongArch64),
      _ => Err(
        "Only supports aarch/arm64, arm/arm32, x86_64/x64 and loongarch64 architectures."
          .to_string(),
      ),
    }
  }
}
