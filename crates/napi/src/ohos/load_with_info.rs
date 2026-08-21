use std::ffi::CString;

use crate::Result;

/// Same signature as [`super::ark::ArkRuntime::load_with_info`].
///
/// Extracted so `path` and `module_info` can be independent `AsRef<str>`
/// types (`&str` vs `String`) and so host tests can exercise that without
/// an Ark runtime or Harmony device.
pub(crate) fn load_with_info<P, I>(path: P, module_info: I) -> Result<(CString, CString)>
where
  P: AsRef<str>,
  I: AsRef<str>,
{
  let c_path = CString::new(path.as_ref())?;
  let c_info = CString::new(module_info.as_ref())?;
  Ok((c_path, c_info))
}

#[cfg(test)]
mod tests {
  use super::load_with_info;

  #[test]
  fn accepts_mixed_str_and_string() {
    let path = "ets/Test";
    let bundle = "com.example.application";
    let entry = "entry";
    let (c_path, c_info) = load_with_info(path, format!("{bundle}/{entry}")).unwrap();
    assert_eq!(c_path.as_bytes(), b"ets/Test");
    assert_eq!(c_info.as_bytes(), b"com.example.application/entry");
  }
}
