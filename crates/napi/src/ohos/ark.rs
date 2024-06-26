use std::{ffi::CString, ptr};

use crate::{check_pending_exception, check_status, Env, Result};

use super::module::Module;

pub struct ArkRuntime {
  pub env: Env,
  is_new: bool,
}

impl ArkRuntime {
  /// create new arkts runtime
  pub fn new() -> Result<Self> {
    let mut env = std::ptr::null_mut();
    check_status!(
      unsafe { napi_sys_ohos::napi_create_ark_runtime(&mut env) },
      "Create arkts runtime failed"
    )?;
    Ok(Self {
      env: Env::from_raw(env),
      is_new: true,
    })
  }

  /// create with existed arkts runtime for example main thread
  pub fn new_with_env(env: Env) -> Self {
    Self {
      env: env,
      is_new: false,
    }
  }

  /// note: This method only can call in main thread, the runtime must be initialized with `new_with_env`
  pub fn load<T: AsRef<str>>(&self, path: T) -> Result<Module> {
    let c_path = CString::new(path.as_ref())?;
    let mut module = ptr::null_mut();
    check_pending_exception!(self.env.0, unsafe {
      napi_sys_ohos::napi_load_module(self.env.0, c_path.as_ptr(), &mut module)
    })?;
    Ok(Module::new(self.env.0, module))
  }

  pub fn load_with_info<T: AsRef<str>>(&self, path: T, module_info: T) -> Result<Module> {
    let c_path = CString::new(path.as_ref())?;
    let c_info = CString::new(module_info.as_ref())?;
    let mut module = ptr::null_mut();
    check_pending_exception!(self.env.0, unsafe {
      napi_sys_ohos::napi_load_module_with_info(
        self.env.0,
        c_path.as_ptr(),
        c_info.as_ptr(),
        &mut module,
      )
    })?;
    Ok(Module::new(self.env.0, module))
  }
}

impl Drop for ArkRuntime {
  fn drop(&mut self) {
    if self.is_new {
      unsafe {
        napi_sys_ohos::napi_destroy_ark_runtime(&mut self.env.0);
      }
    }
  }
}
