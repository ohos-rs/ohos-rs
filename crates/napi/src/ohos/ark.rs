use std::{ffi::CString, ptr};

use super::module::Module;
use crate::{check_pending_exception, check_status, Env, Result};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventLoopMode {
  NonBlocking,
  Blocking,
}

impl From<EventLoopMode> for napi_sys_ohos::napi_event_mode {
  fn from(value: EventLoopMode) -> Self {
    match value {
      EventLoopMode::Blocking => napi_sys_ohos::napi_event_mode::napi_event_mode_default,
      EventLoopMode::NonBlocking => napi_sys_ohos::napi_event_mode::napi_event_mode_nowait,
    }
  }
}

/// Create a new virtual machine for ArkTS, we can use it to call some ArkTS method with rust code   
/// ArkRuntime is not thread safe, so we should not use it in multi-thread. Every ark runtime instance should has its own thread.   
/// See more: https://developer.huawei.com/consumer/cn/doc/harmonyos-references-V5/napi-V5#napi_create_ark_runtime
pub struct ArkRuntime {
  pub env: Env,
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
    })
  }

  /// try to start current env's event loop
  pub fn run_loop(&self, mode: EventLoopMode) -> Result<()> {
    check_status!(
      unsafe { napi_sys_ohos::napi_run_event_loop(self.env.0, mode.into()) },
      "Start event loop failed."
    )?;
    Ok(())
  }

  /// try to start current env's event loop
  pub fn stop_loop(&self) -> Result<()> {
    check_status!(
      unsafe { napi_sys_ohos::napi_stop_event_loop(self.env.0) },
      "Stop event loop failed."
    )?;
    Ok(())
  }

  /// Same with load_with_info, but we don't need module_info.It uses to load built-in module.
  /// ```
  /// #[napi]
  /// pub fn run_ble() -> Result<JsNumber> {
  ///   let runtime = ArkRuntime::new()?;
  ///   let module = runtime.load_without_info("@kit.ConnectivityKit")?;
  ///
  ///   let access: Module = module.get("access")?;
  ///   let ret = access.call_without_args("getState")?;
  ///   ret.coerce_to_number()
  /// }
  /// ```
  pub fn load_without_info<T: AsRef<str>>(&self, path: T) -> Result<Module> {
    let c_path = CString::new(path.as_ref())?;
    let mut module = ptr::null_mut();
    check_pending_exception!(self.env.0, unsafe {
      napi_sys_ohos::napi_load_module_with_info(
        self.env.0,
        c_path.as_ptr().cast(),
        ptr::null(),
        &mut module,
      )
    })?;
    Ok(Module::new(self.env.0, module))
  }

  /// [napi_load_module_with_info](https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/use-napi-load-module-with-info-V5#napi_load_module_with_info) load module with har/hsp module info.
  /// ```
  /// #[napi]
  /// pub fn run_ble() -> Result<JsNumber> {
  ///   let runtime = ArkRuntime::new()?;
  ///   let module = runtime.load_with_info("entry/src/main/ets/Test", "com.example.application/entry")?;
  ///
  ///   let access: Module = module.get("access")?;
  ///   let ret = access.call_without_args("getState")?;
  ///   ret.coerce_to_number()
  /// }
  /// ```
  pub fn load_with_info<T: AsRef<str>>(&self, path: T, module_info: T) -> Result<Module> {
    let c_path = CString::new(path.as_ref())?;
    let c_info = CString::new(module_info.as_ref())?;
    let mut module = ptr::null_mut();
    check_pending_exception!(self.env.0, unsafe {
      napi_sys_ohos::napi_load_module_with_info(
        self.env.0,
        c_path.as_ptr().cast(),
        c_info.as_ptr().cast(),
        &mut module,
      )
    })?;
    Ok(Module::new(self.env.0, module))
  }
}

impl Drop for ArkRuntime {
  fn drop(&mut self) {
    unsafe {
      napi_sys_ohos::napi_destroy_ark_runtime(&mut self.env.0);
    }
  }
}
