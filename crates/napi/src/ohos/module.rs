use napi_sys_ohos::{napi_env, napi_value};
use std::{ffi::CString, ptr};

use crate::{
  bindgen_runtime::{FromNapiValue, ToNapiValue},
  check_pending_exception, check_status, type_of, Error, JsUnknown, NapiRaw, NapiValue, Result,
  Status, ValueType,
};

pub struct Module {
  env: napi_env,
  // module
  value: napi_value,
}

// allow as nest module
impl FromNapiValue for Module {
  unsafe fn from_napi_value(
    env: napi_sys_ohos::napi_env,
    napi_val: napi_sys_ohos::napi_value,
  ) -> Result<Self> {
    Ok(Module {
      env,
      value: napi_val,
    })
  }
}

impl ToNapiValue for Module {
  unsafe fn to_napi_value(
    _env: napi_sys_ohos::napi_env,
    val: Self,
  ) -> Result<napi_sys_ohos::napi_value> {
    Ok(val.value)
  }
}

/// load module with napi_load_module/napi_load_module_with_info
impl Module {
  pub fn new(env: napi_env, module: napi_value) -> Self {
    Self { env, value: module }
  }

  pub fn get_napi_value<K: AsRef<str>>(&self, property: K) -> Result<napi_value> {
    let c_field = CString::new(property.as_ref())?;

    unsafe {
      let mut ret = ptr::null_mut();

      check_status!(
        napi_sys_ohos::napi_get_named_property(self.env, self.value, c_field.as_ptr(), &mut ret),
        "Failed to get property with field `{}`",
        property.as_ref(),
      )?;

      Ok(ret)
    }
  }

  pub fn get<K: AsRef<str>, V: FromNapiValue>(&self, property: K) -> Result<V> {
    let ret = self.get_napi_value(property)?;
    unsafe { Ok(V::from_napi_value(self.env, ret)?) }
  }

  /// [napi_call_function](https://nodejs.org/api/n-api.html#n_api_napi_call_function)
  pub fn call<K, V>(&self, property: K, args: &[V]) -> Result<JsUnknown>
  where
    V: NapiRaw,
    K: AsRef<str>,
  {
    let ret = self.get_napi_value(property.as_ref())?;
    let ty = type_of!(self.env, ret)?;

    if ty != ValueType::Function {
      return Err(Error::new(
        Status::GenericFailure,
        format!("{} is not callable", property.as_ref()),
      ));
    }

    let raw_args = args
      .iter()
      .map(|arg| unsafe { arg.raw() })
      .collect::<Vec<napi_sys_ohos::napi_value>>();
    let mut return_value = ptr::null_mut();
    check_pending_exception!(self.env, unsafe {
      napi_sys_ohos::napi_call_function(
        self.env,
        self.value,
        ret,
        args.len(),
        raw_args.as_ptr(),
        &mut return_value,
      )
    })?;

    unsafe { JsUnknown::from_raw(self.env, return_value) }
  }

  /// [napi_call_function](https://nodejs.org/api/n-api.html#n_api_napi_call_function)
  /// The same with `call`, but without arguments
  pub fn call_without_args<K: AsRef<str>>(&self, property: K) -> Result<JsUnknown> {
    let ret = self.get_napi_value(property.as_ref())?;
    let ty = type_of!(self.env, ret)?;

    if ty != ValueType::Function {
      return Err(Error::new(
        Status::GenericFailure,
        format!("{} is not callable", property.as_ref()),
      ));
    }
    let mut return_value = ptr::null_mut();
    check_pending_exception!(self.env, unsafe {
      napi_sys_ohos::napi_call_function(
        self.env,
        self.value,
        ret,
        0,
        ptr::null_mut(),
        &mut return_value,
      )
    })?;

    unsafe { JsUnknown::from_raw(self.env, return_value) }
  }
}
