use std::ffi::{c_void, CString};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
#[cfg(all(feature = "napi4", not(feature = "noop")))]
use std::sync::atomic::Ordering;

use crate::bindgen_prelude::This;
#[cfg(all(feature = "napi4", not(feature = "noop")))]
use crate::bindgen_prelude::{CUSTOM_GC_TSFN, CUSTOM_GC_TSFN_DESTROYED, THREADS_CAN_ACCESS_ENV};
pub use crate::js_values::TypedArrayType;
use crate::{check_status, sys, Env, Error, NapiRaw, Result, Status, ValueType};

use super::{FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue};

#[cfg(target_family = "wasm")]
extern "C" {
  fn emnapi_sync_memory(
    env: crate::sys::napi_env,
    js_to_wasm: bool,
    arraybuffer_or_view: crate::sys::napi_value,
    byte_offset: usize,
    length: usize,
  ) -> crate::sys::napi_status;
}

trait Finalizer {
  type RustType;

  fn finalizer_notify(&self) -> *mut dyn FnOnce(*mut Self::RustType, usize);
}

macro_rules! impl_typed_array {
  ($name:ident, $rust_type:ident, $typed_array_type:expr) => {
    pub struct $name {
      data: *mut $rust_type,
      length: usize,
      #[allow(unused)]
      byte_offset: usize,
      raw: Option<(crate::sys::napi_ref, crate::sys::napi_env)>,
      finalizer_notify: *mut dyn FnOnce(*mut $rust_type, usize),
    }

    /// SAFETY: This is undefined behavior, as the JS side may always modify the underlying buffer,
    /// without synchronization. Also see the docs for the `DerfMut` impl.
    unsafe impl Send for $name {}
    unsafe impl Sync for $name {}

    impl Finalizer for $name {
      type RustType = $rust_type;

      fn finalizer_notify(&self) -> *mut dyn FnOnce(*mut Self::RustType, usize) {
        self.finalizer_notify
      }
    }

    impl Drop for $name {
      fn drop(&mut self) {
        if let Some((ref_, env)) = self.raw {
          // If the ref is null, it means the TypedArray has been called `ToNapiValue::to_napi_value`, and the `ref` has been deleted
          // If the env is null, it means the TypedArray is copied in `&mut TypedArray ToNapiValue::to_napi_value`, and the `ref` will be deleted in the raw TypedArray
          if ref_.is_null() || env.is_null() {
            return;
          }
          #[cfg(all(feature = "napi4", not(feature = "noop")))]
          {
            if CUSTOM_GC_TSFN_DESTROYED.load(Ordering::SeqCst) {
              return;
            }
            if !THREADS_CAN_ACCESS_ENV.borrow_mut(|m| m.get(&std::thread::current().id()).is_some())
            {
              let status = unsafe {
                sys::napi_call_threadsafe_function(
                  CUSTOM_GC_TSFN.load(std::sync::atomic::Ordering::SeqCst),
                  ref_.cast(),
                  1,
                )
              };
              assert!(
                status == sys::Status::napi_ok || status == sys::Status::napi_closing,
                "Call custom GC in ArrayBuffer::drop failed {}",
                Status::from(status)
              );
              return;
            }
          }
          let mut ref_count = 0;
          crate::check_status_or_throw!(
            env,
            unsafe { sys::napi_reference_unref(env, ref_, &mut ref_count) },
            "Failed to unref ArrayBuffer reference in drop"
          );
          debug_assert!(
            ref_count == 0,
            "ArrayBuffer reference count in ArrayBuffer::drop is not zero"
          );
          crate::check_status_or_throw!(
            env,
            unsafe { sys::napi_delete_reference(env, ref_) },
            "Failed to delete ArrayBuffer reference in drop"
          );
          return;
        }
        if !self.data.is_null() {
          let length = self.length;
          unsafe { Vec::from_raw_parts(self.data, length, length) };
          return;
        }
        if !self.finalizer_notify().is_null() {
          let finalizer = unsafe { Box::from_raw(self.finalizer_notify) };
          (finalizer)(self.data, self.length);
        }
      }
    }

    impl $name {
      #[cfg(target_family = "wasm")]
      pub fn sync(&mut self, env: &crate::Env) {
        if let Some((reference, _)) = self.raw {
          let mut value = ptr::null_mut();
          let mut array_buffer = ptr::null_mut();
          crate::check_status_or_throw!(
            env.raw(),
            unsafe { crate::sys::napi_get_reference_value(env.raw(), reference, &mut value) },
            "Failed to get reference value from TypedArray while syncing"
          );
          crate::check_status_or_throw!(
            env.raw(),
            unsafe {
              crate::sys::napi_get_typedarray_info(
                env.raw(),
                value,
                &mut ($typed_array_type as i32) as *mut i32,
                &mut self.length as *mut usize,
                ptr::null_mut(),
                &mut array_buffer,
                &mut self.byte_offset as *mut usize,
              )
            },
            "Failed to get ArrayBuffer under the TypedArray while syncing"
          );
          crate::check_status_or_throw!(
            env.raw(),
            unsafe {
              emnapi_sync_memory(
                env.raw(),
                false,
                array_buffer,
                self.byte_offset,
                self.length,
              )
            },
            "Failed to sync memory"
          );
        } else {
          return;
        }
      }

      pub fn new(mut data: Vec<$rust_type>) -> Self {
        data.shrink_to_fit();
        let ret = $name {
          data: data.as_mut_ptr(),
          length: data.len(),
          byte_offset: 0,
          raw: None,
          finalizer_notify: ptr::null_mut::<fn(*mut $rust_type, usize)>(),
        };
        mem::forget(data);
        ret
      }

      pub fn with_data_copied<D>(data: D) -> Self
      where
        D: AsRef<[$rust_type]>,
      {
        let mut data_copied = data.as_ref().to_vec();
        let ret = $name {
          data: data_copied.as_mut_ptr(),
          length: data.as_ref().len(),
          finalizer_notify: ptr::null_mut::<fn(*mut $rust_type, usize)>(),
          raw: None,
          byte_offset: 0,
        };
        mem::forget(data_copied);
        ret
      }

      /// # Safety
      ///
      /// The caller will be notified when the data is deallocated by vm
      pub unsafe fn with_external_data<F>(data: *mut $rust_type, length: usize, notify: F) -> Self
      where
        F: 'static + FnOnce(*mut $rust_type, usize),
      {
        $name {
          data,
          length,
          finalizer_notify: Box::into_raw(Box::new(notify)),
          raw: None,
          byte_offset: 0,
        }
      }
    }

    impl Deref for $name {
      type Target = [$rust_type];

      fn deref(&self) -> &Self::Target {
        self.as_ref()
      }
    }

    /// SAFETY: This is literally undefined behavior. `Buffer::clone` allows you to create shared
    /// access to the underlying data, but `as_mut` and `deref_mut` allow unsynchronized mutation of
    /// that data (not to speak of the JS side having write access as well, at the same time).
    impl DerefMut for $name {
      fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
      }
    }

    impl AsRef<[$rust_type]> for $name {
      fn as_ref(&self) -> &[$rust_type] {
        if self.data.is_null() {
          return &[];
        }

        unsafe { std::slice::from_raw_parts(self.data, self.length) }
      }
    }

    impl AsMut<[$rust_type]> for $name {
      fn as_mut(&mut self) -> &mut [$rust_type] {
        if self.data.is_null() {
          return &mut [];
        }

        unsafe { std::slice::from_raw_parts_mut(self.data, self.length) }
      }
    }

    impl TypeName for $name {
      fn type_name() -> &'static str {
        concat!("TypedArray<", stringify!($rust_type), ">")
      }

      fn value_type() -> crate::ValueType {
        crate::ValueType::Object
      }
    }

    impl ValidateNapiValue for $name {
      unsafe fn validate(
        env: sys::napi_env,
        napi_val: sys::napi_value,
      ) -> Result<crate::sys::napi_value> {
        let mut is_typed_array = false;
        check_status!(
          unsafe { sys::napi_is_typedarray(env, napi_val, &mut is_typed_array) },
          "Failed to check if value is typed array"
        )?;
        if !is_typed_array {
          return Err(Error::new(
            Status::InvalidArg,
            "Expected a TypedArray value".to_owned(),
          ));
        }
        Ok(ptr::null_mut())
      }
    }

    impl FromNapiValue for $name {
      unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
        let mut typed_array_type = 0;
        let mut length = 0;
        let mut data = ptr::null_mut();
        let mut array_buffer = ptr::null_mut();
        let mut byte_offset = 0;
        let mut ref_ = ptr::null_mut();
        check_status!(
          unsafe { sys::napi_create_reference(env, napi_val, 1, &mut ref_) },
          "Failed to create reference from Buffer"
        )?;
        check_status!(
          unsafe {
            sys::napi_get_typedarray_info(
              env,
              napi_val,
              &mut typed_array_type,
              &mut length,
              &mut data,
              &mut array_buffer,
              &mut byte_offset,
            )
          },
          "Get TypedArray info failed"
        )?;
        if typed_array_type != $typed_array_type as i32 {
          return Err(Error::new(
            Status::InvalidArg,
            format!(
              "Expected {}, got {}Array",
              stringify!($name),
              TypedArrayType::from(typed_array_type).as_ref()
            ),
          ));
        }
        length /= $typed_array_type.byte_len();
        Ok($name {
          data: data.cast(),
          length,
          byte_offset,
          raw: Some((ref_, env)),
          finalizer_notify: ptr::null_mut::<fn(*mut $rust_type, usize)>(),
        })
      }
    }

    impl ToNapiValue for $name {
      unsafe fn to_napi_value(env: sys::napi_env, mut val: Self) -> Result<sys::napi_value> {
        if let Some((ref_, _)) = val.raw {
          let mut napi_value = std::ptr::null_mut();
          check_status!(
            unsafe { sys::napi_get_reference_value(env, ref_, &mut napi_value) },
            "Failed to get reference from ArrayBuffer"
          )?;
          check_status!(
            unsafe { sys::napi_delete_reference(env, ref_) },
            "Failed to delete reference in ArrayBuffer::to_napi_value"
          )?;
          val.raw = Some((ptr::null_mut(), ptr::null_mut()));
          return Ok(napi_value);
        }
        let mut arraybuffer_value = ptr::null_mut();
        let ratio = mem::size_of::<$rust_type>();
        let val_length = val.length;
        let length = val_length * ratio;
        let val_data = val.data;
        check_status!(
          if length == 0 {
            // Rust uses 0x1 as the data pointer for empty buffers,
            // but NAPI/V8 only allows multiple buffers to have
            // the same data pointer if it's 0x0.
            // For harmony we can't use ptr::null_mut() directly
            let mut empty_data = ptr::null_mut();
            unsafe {
              sys::napi_create_arraybuffer(env, length, &mut empty_data, &mut arraybuffer_value)
            }
          } else {
            let hint_ptr = Box::into_raw(Box::new(val));
            let status = unsafe {
              sys::napi_create_external_arraybuffer(
                env,
                val_data.cast(),
                length,
                Some(finalizer::<$rust_type, $name>),
                hint_ptr.cast(),
                &mut arraybuffer_value,
              )
            };
            if status == napi_sys_ohos::Status::napi_no_external_buffers_allowed {
              let hint = unsafe { Box::from_raw(hint_ptr) };
              let mut underlying_data = ptr::null_mut();
              let status = unsafe {
                sys::napi_create_arraybuffer(
                  env,
                  length,
                  &mut underlying_data,
                  &mut arraybuffer_value,
                )
              };
              unsafe { std::ptr::copy_nonoverlapping(hint.data.cast(), underlying_data, length) };
              status
            } else {
              status
            }
          },
          "Create external arraybuffer failed"
        )?;
        let mut napi_val = ptr::null_mut();
        check_status!(
          unsafe {
            sys::napi_create_typedarray(
              env,
              $typed_array_type as i32,
              val_length,
              arraybuffer_value,
              0,
              &mut napi_val,
            )
          },
          "Create TypedArray failed"
        )?;
        Ok(napi_val)
      }
    }

    impl ToNapiValue for &mut $name {
      unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
        if let Some((ref_, _)) = val.raw {
          let mut napi_value = std::ptr::null_mut();
          check_status!(
            unsafe { sys::napi_get_reference_value(env, ref_, &mut napi_value) },
            "Failed to get reference from ArrayBuffer"
          )?;
          return Ok(napi_value);
        }
        let mut arraybuffer_value = ptr::null_mut();
        let ratio = mem::size_of::<$rust_type>();
        let val_length = val.length;
        let length = val_length * ratio;
        let val_data = val.data;
        let mut copied_val = None;
        check_status!(
          if length == 0 {
            // Rust uses 0x1 as the data pointer for empty buffers,
            // but NAPI/V8 only allows multiple buffers to have
            // the same data pointer if it's 0x0.
            unsafe {
              sys::napi_create_arraybuffer(env, length, ptr::null_mut(), &mut arraybuffer_value)
            }
          } else {
            // manually copy the data instead of implement `Clone` & `Copy` for TypedArray
            // the TypedArray can't be copied if raw is not None
            let val_copy = $name {
              data: val.data,
              length: val.length,
              byte_offset: val.byte_offset,
              raw: None,
              finalizer_notify: val.finalizer_notify,
            };
            let hint_ref: &mut $name = Box::leak(Box::new(val_copy));
            let hint_ptr = hint_ref as *mut $name;
            copied_val = Some(hint_ref);
            let status = unsafe {
              sys::napi_create_external_arraybuffer(
                env,
                val_data.cast(),
                length,
                Some(finalizer::<$rust_type, $name>),
                hint_ptr.cast(),
                &mut arraybuffer_value,
              )
            };
            if status == napi_sys_ohos::Status::napi_no_external_buffers_allowed {
              let hint = unsafe { Box::from_raw(hint_ptr) };
              let mut underlying_data = ptr::null_mut();
              let status = unsafe {
                sys::napi_create_arraybuffer(
                  env,
                  length,
                  &mut underlying_data,
                  &mut arraybuffer_value,
                )
              };
              unsafe { std::ptr::copy_nonoverlapping(hint.data.cast(), underlying_data, length) };
              status
            } else {
              status
            }
          },
          "Create external arraybuffer failed"
        )?;
        let mut napi_val = ptr::null_mut();
        check_status!(
          unsafe {
            sys::napi_create_typedarray(
              env,
              $typed_array_type as i32,
              val_length,
              arraybuffer_value,
              0,
              &mut napi_val,
            )
          },
          "Create TypedArray failed"
        )?;
        let mut ref_ = ptr::null_mut();
        check_status!(
          unsafe { sys::napi_create_reference(env, napi_val, 1, &mut ref_) },
          "Failed to delete reference in ArrayBuffer::to_napi_value"
        )?;
        val.raw = Some((ref_, env));
        if let Some(copied_val) = copied_val {
          val.finalizer_notify = ptr::null_mut::<fn(*mut $rust_type, usize)>();
          val.data = ptr::null_mut();
          val.length = 0;
          copied_val.raw = Some((ref_, ptr::null_mut()));
        }
        Ok(napi_val)
      }
    }
  };
}

macro_rules! impl_from_slice {
  ($name:ident, $slice_type:ident, $rust_type:ident, $typed_array_type:expr) => {
    pub struct $slice_type<'scope> {
      pub(crate) inner: &'scope mut [$rust_type],
      raw_value: sys::napi_value,
      env: sys::napi_env,
    }

    impl <'scope> $slice_type<'scope> {
      #[doc = " Create a new `"]
      #[doc = stringify!($slice_type)]
      #[doc = "` from a `Vec<"]
      #[doc = stringify!($rust_type)]
      #[doc = ">`."]
      pub fn from_data<D: Into<Vec<u8>>>(env: &Env, data: D) -> Result<Self> {
        let mut buf = ptr::null_mut();
        let mut data = data.into();
        let mut inner_ptr = data.as_mut_ptr();
        #[cfg(all(debug_assertions, not(windows)))]
        {
          let is_existed = super::BUFFER_DATA.with(|buffer_data| {
            let buffer = buffer_data.lock().expect("Unlock buffer data failed");
            buffer.contains(&inner_ptr)
          });
          if is_existed {
            panic!("Share the same data between different buffers is not allowed, see: https://github.com/nodejs/node/issues/32463#issuecomment-631974747");
          }
        }
        let len = data.len();
        let mut status = unsafe {
          sys::napi_create_external_arraybuffer(
            env.0,
            inner_ptr.cast(),
            data.len(),
            Some(finalize_slice::<$rust_type>),
            Box::into_raw(Box::new(len)).cast(),
            &mut buf,
          )
        };
        if status == napi_sys_ohos::Status::napi_no_external_buffers_allowed {
          let mut inner_data = unsafe { Vec::from_raw_parts(inner_ptr, data.len(), data.len()) };
          let mut underlying_data = ptr::null_mut();
          status = unsafe {
            sys::napi_create_arraybuffer(
              env.0,
              data.len(),
              &mut underlying_data,
              &mut buf,
            )
          };
          unsafe { std::ptr::copy_nonoverlapping(inner_data.as_mut_ptr().cast(), underlying_data, data.len()) };
          inner_ptr = underlying_data.cast();
        } else {
          mem::forget(data);
        }
        check_status!(status, "Failed to create buffer slice from data")?;

        let mut napi_val = ptr::null_mut();
        check_status!(
          unsafe {
            sys::napi_create_typedarray(
              env.0,
              $typed_array_type as i32,
              len,
              buf,
              0,
              &mut napi_val,
            )
          },
          "Create TypedArray failed"
        )?;

        Ok(Self {
          inner: if len == 0 {
            &mut []
          } else {
            unsafe { core::slice::from_raw_parts_mut(inner_ptr.cast(), len) }
          },
          raw_value: napi_val,
          env: env.0,
        })
      }

      #[doc = "## Safety"]
      #[doc = ""]
      #[doc = "Mostly the same with `from_data`"]
      #[doc = ""]
      #[doc = "Provided `finalize_callback` will be called when `"]
      #[doc = stringify!($slice_type)]
      #[doc = "` got dropped."]
      #[doc = ""]
      #[doc = "You can pass in `noop_finalize` if you have nothing to do in finalize phase."]
      #[doc = ""]
      #[doc = "### Notes"]
      #[doc = ""]
      #[doc = "JavaScript may mutate the data passed in to this buffer when writing the buffer."]
      #[doc = "However, some JavaScript runtimes do not support external buffers (notably electron!)"]
      #[doc = "in which case modifications may be lost."]
      #[doc = ""]
      #[doc = "If you need to support these runtimes, you should create a buffer by other means and then"]
      #[doc = "later copy the data back out."]
      pub unsafe fn from_external<T: 'scope, F: FnOnce(Env, T)>(
        env: &Env,
        data: *mut u8,
        len: usize,
        finalize_hint: T,
        finalize_callback: F,
      ) -> Result<Self> {
        if data.is_null() || data as *const u8 == crate::EMPTY_VEC.as_ptr() {
          return Err(Error::new(
            Status::InvalidArg,
            "Borrowed data should not be null".to_owned(),
          ));
        }
        #[cfg(all(debug_assertions, not(windows)))]
        {
          let is_existed = super::BUFFER_DATA.with(|buffer_data| {
            let buffer = buffer_data.lock().expect("Unlock buffer data failed");
            buffer.contains(&data)
          });
          if is_existed {
            panic!("Share the same data between different buffers is not allowed, see: https://github.com/nodejs/node/issues/32463#issuecomment-631974747");
          }
        }
        let hint_ptr = Box::into_raw(Box::new((finalize_hint, finalize_callback)));
        let mut arraybuffer_value = ptr::null_mut();
        let mut status = unsafe {
          sys::napi_create_external_arraybuffer(
            env.0,
            data.cast(),
            len,
            Some(crate::env::raw_finalize_with_custom_callback::<T, F>),
            hint_ptr.cast(),
            &mut arraybuffer_value,
          )
        };
        status = if status == sys::Status::napi_no_external_buffers_allowed {
          let (hint, finalize) = *Box::from_raw(hint_ptr);
          let mut underlying_data = ptr::null_mut();
          let status = unsafe {
            sys::napi_create_arraybuffer(
              env.0,
              len,
              &mut underlying_data,
              &mut arraybuffer_value,
            )
          };
          unsafe { std::ptr::copy_nonoverlapping(data.cast(), underlying_data, len) };
          finalize(*env, hint);
          status
        } else {
          status
        };
        check_status!(status, "Failed to create arraybuffer from data")?;

        let mut napi_val = ptr::null_mut();
        check_status!(
          unsafe {
            sys::napi_create_typedarray(
              env.0,
              $typed_array_type as i32,
              len,
              arraybuffer_value,
              0,
              &mut napi_val,
            )
          },
          "Create TypedArray failed"
        )?;

        Ok(Self {
          inner: if len == 0 {
            &mut []
          } else {
            unsafe { core::slice::from_raw_parts_mut(data.cast(), len) }
          },
          raw_value: napi_val,
          env: env.0,
        })
      }

      #[doc = "Copy data from a `&["]
      #[doc = stringify!($rust_type)]
      #[doc = "]` and create a `"]
      #[doc = stringify!($slice_type)]
      #[doc = "` from it."]
      pub fn copy_from<D: AsRef<[$rust_type]>>(env: &Env, data: D) -> Result<Self> {
        let data = data.as_ref();
        let len = data.len();
        let mut arraybuffer_value = ptr::null_mut();
        let mut underlying_data = ptr::null_mut();

        check_status!(
          unsafe {
            sys::napi_create_arraybuffer(
              env.0,
              len,
              &mut underlying_data,
              &mut arraybuffer_value,
            )
          },
          "Failed to create ArrayBuffer"
        )?;

        let mut napi_val = ptr::null_mut();
        check_status!(
          unsafe {
            sys::napi_create_typedarray(
              env.0,
              $typed_array_type as i32,
              len,
              arraybuffer_value,
              0,
              &mut napi_val,
            )
          },
          "Create TypedArray failed"
        )?;

        Ok(Self {
          inner: if len == 0 {
            &mut []
          } else {
            unsafe { core::slice::from_raw_parts_mut(underlying_data.cast(), len) }
          },
          raw_value: napi_val,
          env: env.0,
        })
      }

      pub fn assign_to_this<'a, U>(&self, this: This<'a, U>, name: &str) -> Result<$slice_type<'a>>
      where
        U: FromNapiValue + NapiRaw,
      {
        let name = CString::new(name)?;
        check_status!(
          unsafe { sys::napi_set_named_property(self.env, this.object.raw(), name.as_ptr(), self.raw_value) },
          "Failed to assign {} to this",
          $slice_type::type_name()
        )?;
        let inner: &'a mut [$rust_type] = unsafe { core::slice::from_raw_parts_mut(self.inner.as_ptr().cast_mut(), self.inner.len()) };
        Ok($slice_type {
          env: self.env,
          raw_value: self.raw_value,
          inner,
        })
      }

      #[doc = "Convert a `"]
      #[doc = stringify!($slice_type)]
      #[doc = "` to a `"]
      #[doc = stringify!($name)]
      #[doc = "`."]
      #[doc = ""]
      #[doc = "This will perform a `napi_create_reference` internally."]
      pub fn into_typed_array(self, env: &Env) -> Result<$name> {
        unsafe { $name::from_napi_value(env.0, self.raw_value) }
      }
    }

    impl NapiRaw for $slice_type<'_> {
      unsafe fn raw(&self) -> napi_sys_ohos::napi_value {
        self.raw_value
      }
    }

    impl ToNapiValue for &$slice_type<'_> {
      unsafe fn to_napi_value(_: sys::napi_env, val: Self) -> Result<sys::napi_value> {
        Ok(val.raw_value)
      }
    }

    impl ToNapiValue for &mut $slice_type<'_> {
      unsafe fn to_napi_value(_: sys::napi_env, val: Self) -> Result<sys::napi_value> {
        Ok(val.raw_value)
      }
    }

    impl FromNapiValue for $slice_type<'_> {
      unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
        let mut typed_array_type = 0;
        let mut length = 0;
        let mut data = ptr::null_mut();
        let mut array_buffer = ptr::null_mut();
        let mut byte_offset = 0;
        check_status!(
          unsafe {
            sys::napi_get_typedarray_info(
              env,
              napi_val,
              &mut typed_array_type,
              &mut length,
              &mut data,
              &mut array_buffer,
              &mut byte_offset,
            )
          },
          "Get TypedArray info failed"
        )?;
        if typed_array_type != $typed_array_type as i32 {
          return Err(Error::new(
            Status::InvalidArg,
            format!("Expected $name, got {}", typed_array_type),
          ));
        }
        // From the docs of `napi_get_typedarray_info`:
        // > [out] data: The underlying data buffer of the node::Buffer. If length is 0, this may be
        // > NULL or any other pointer value.
        //
        // In order to guarantee that `slice::from_raw_parts` is sound, the pointer must be non-null, so
        // let's make sure it always is, even in the case of `napi_get_typedarray_info` returning a null
        // ptr.
        Ok(Self {
          inner: if length == 0 {
            &mut []
          } else {
            unsafe { core::slice::from_raw_parts_mut(data as *mut $rust_type, length) }
          },
          raw_value: napi_val,
          env,
        })
      }
    }

    impl TypeName for $slice_type<'_> {
      fn type_name() -> &'static str {
        concat!("TypedArray<", stringify!($rust_type), ">")
      }

      fn value_type() -> crate::ValueType {
        crate::ValueType::Object
      }
    }

    impl ValidateNapiValue for $slice_type<'_> {
      unsafe fn validate(env: sys::napi_env, napi_val: sys::napi_value) -> Result<sys::napi_value> {
        let mut is_typed_array = false;
        check_status!(
          unsafe { sys::napi_is_typedarray(env, napi_val, &mut is_typed_array) },
          "Failed to validate napi typed array"
        )?;
        if !is_typed_array {
          return Err(Error::new(
            Status::InvalidArg,
            "Expected a TypedArray value".to_owned(),
          ));
        }
        Ok(ptr::null_mut())
      }
    }

    impl AsRef<[$rust_type]> for $slice_type<'_> {
      fn as_ref(&self) -> &[$rust_type] {
        self.inner
      }
    }

    impl Deref for $slice_type<'_> {
      type Target = [$rust_type];

      fn deref(&self) -> &Self::Target {
        self.as_ref()
      }
    }

    impl DerefMut for $slice_type<'_> {
      fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
      }
    }

    impl FromNapiValue for &mut [$rust_type] {
      unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
        let mut typed_array_type = 0;
        let mut length = 0;
        let mut data = ptr::null_mut();
        let mut array_buffer = ptr::null_mut();
        let mut byte_offset = 0;
        check_status!(
          unsafe {
            sys::napi_get_typedarray_info(
              env,
              napi_val,
              &mut typed_array_type,
              &mut length,
              &mut data,
              &mut array_buffer,
              &mut byte_offset,
            )
          },
          "Get TypedArray info failed"
        )?;
        if typed_array_type != $typed_array_type as i32 {
          return Err(Error::new(
            Status::InvalidArg,
            format!("Expected $name, got {}", typed_array_type),
          ));
        }
        length /= $typed_array_type.byte_len();
        Ok(if length == 0 {
          &mut []
        } else {
          unsafe { core::slice::from_raw_parts_mut(data as *mut $rust_type, length) }
        })
      }
    }

    impl FromNapiValue for &[$rust_type] {
      unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
        let mut typed_array_type = 0;
        let mut length = 0;
        let mut data = ptr::null_mut();
        let mut array_buffer = ptr::null_mut();
        let mut byte_offset = 0;
        check_status!(
          unsafe {
            sys::napi_get_typedarray_info(
              env,
              napi_val,
              &mut typed_array_type,
              &mut length,
              &mut data,
              &mut array_buffer,
              &mut byte_offset,
            )
          },
          "Get TypedArray info failed"
        )?;
        if typed_array_type != $typed_array_type as i32 {
          return Err(Error::new(
            Status::InvalidArg,
            format!("Expected $name, got {}", typed_array_type),
          ));
        }
        length /= $typed_array_type.byte_len();
        Ok(if length == 0 {
          &[]
        } else {
          unsafe { core::slice::from_raw_parts_mut(data as *mut $rust_type, length) }
        })
      }
    }

    impl TypeName for &mut [$rust_type] {
      fn type_name() -> &'static str {
        concat!("TypedArray<", stringify!($rust_type), ">")
      }

      fn value_type() -> crate::ValueType {
        crate::ValueType::Object
      }
    }

    impl TypeName for &[$rust_type] {
      fn type_name() -> &'static str {
        concat!("TypedArray<", stringify!($rust_type), ">")
      }

      fn value_type() -> crate::ValueType {
        crate::ValueType::Object
      }
    }

    impl ValidateNapiValue for &[$rust_type] {
      unsafe fn validate(env: sys::napi_env, napi_val: sys::napi_value) -> Result<sys::napi_value> {
        let mut is_typed_array = false;
        check_status!(
          unsafe { sys::napi_is_typedarray(env, napi_val, &mut is_typed_array) },
          "Failed to validate napi typed array"
        )?;
        if !is_typed_array {
          return Err(Error::new(
            Status::InvalidArg,
            "Expected a TypedArray value".to_owned(),
          ));
        }
        Ok(ptr::null_mut())
      }
    }

    impl ValidateNapiValue for &mut [$rust_type] {
      unsafe fn validate(env: sys::napi_env, napi_val: sys::napi_value) -> Result<sys::napi_value> {
        let mut is_typed_array = false;
        check_status!(
          unsafe { sys::napi_is_typedarray(env, napi_val, &mut is_typed_array) },
          "Failed to validate napi typed array"
        )?;
        if !is_typed_array {
          return Err(Error::new(
            Status::InvalidArg,
            "Expected a TypedArray value".to_owned(),
          ));
        }
        Ok(ptr::null_mut())
      }
    }
  };
}

unsafe extern "C" fn finalizer<Data, T: Finalizer<RustType = Data> + AsRef<[Data]>>(
  _env: sys::napi_env,
  _finalize_data: *mut c_void,
  finalize_hint: *mut c_void,
) {
  let data = unsafe { *Box::from_raw(finalize_hint as *mut T) };
  drop(data);
}

unsafe extern "C" fn finalize_slice<Data>(
  _env: sys::napi_env,
  finalize_data: *mut c_void,
  finalize_hint: *mut c_void,
) {
  let length = unsafe { *Box::from_raw(finalize_hint as *mut usize) };
  unsafe { Vec::from_raw_parts(finalize_data as *mut Data, length, length) };
}

impl_typed_array!(Int8Array, i8, TypedArrayType::Int8);
impl_from_slice!(Int8Array, Int8ArraySlice, i8, TypedArrayType::Int8);
impl_typed_array!(Uint8Array, u8, TypedArrayType::Uint8);
impl_from_slice!(Uint8Array, Uint8ArraySlice, u8, TypedArrayType::Uint8);
impl_typed_array!(Uint8ClampedArray, u8, TypedArrayType::Uint8Clamped);
impl_typed_array!(Int16Array, i16, TypedArrayType::Int16);
impl_from_slice!(Int16Array, Int16ArraySlice, i16, TypedArrayType::Int16);
impl_typed_array!(Uint16Array, u16, TypedArrayType::Uint16);
impl_from_slice!(Uint16Array, Uint16ArraySlice, u16, TypedArrayType::Uint16);
impl_typed_array!(Int32Array, i32, TypedArrayType::Int32);
impl_from_slice!(Int32Array, Int32ArraySlice, i32, TypedArrayType::Int32);
impl_typed_array!(Uint32Array, u32, TypedArrayType::Uint32);
impl_from_slice!(Uint32Array, Uint32ArraySlice, u32, TypedArrayType::Uint32);
impl_typed_array!(Float32Array, f32, TypedArrayType::Float32);
impl_from_slice!(
  Float32Array,
  Float32ArraySlice,
  f32,
  TypedArrayType::Float32
);
impl_typed_array!(Float64Array, f64, TypedArrayType::Float64);
impl_from_slice!(
  Float64Array,
  Float64ArraySlice,
  f64,
  TypedArrayType::Float64
);
#[cfg(feature = "napi6")]
impl_typed_array!(BigInt64Array, i64, TypedArrayType::BigInt64);
#[cfg(feature = "napi6")]
impl_from_slice!(
  BigInt64Array,
  BigInt64ArraySlice,
  i64,
  TypedArrayType::BigInt64
);
#[cfg(feature = "napi6")]
impl_typed_array!(BigUint64Array, u64, TypedArrayType::BigUint64);
#[cfg(feature = "napi6")]
impl_from_slice!(
  BigUint64Array,
  BigUint64ArraySlice,
  u64,
  TypedArrayType::BigUint64
);

impl Uint8Array {
  /// Create a new JavaScript `Uint8Array` from a Rust `String` without copying the underlying data.
  pub fn from_string(mut s: String) -> Self {
    let len = s.len();
    let ret = Self {
      data: s.as_mut_ptr(),
      length: len,
      finalizer_notify: Box::into_raw(Box::new(move |data, _| {
        drop(unsafe { String::from_raw_parts(data, len, len) });
      })),
      byte_offset: 0,
      raw: None,
    };
    mem::forget(s);
    ret
  }
}

/// Zero copy Uint8ClampedArray slice shared between Rust and Node.js.
/// It can only be used in non-async context and the lifetime is bound to the fn closure.
/// If you want to use Node.js `Uint8ClampedArray` in async context or want to extend the lifetime, use `Uint8ClampedArray` instead.
pub struct Uint8ClampedSlice<'scope> {
  pub(crate) inner: &'scope mut [u8],
  raw_value: sys::napi_value,
}

impl FromNapiValue for Uint8ClampedSlice<'_> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let mut typed_array_type = 0;
    let mut length = 0;
    let mut data = ptr::null_mut();
    let mut array_buffer = ptr::null_mut();
    let mut byte_offset = 0;
    check_status!(
      unsafe {
        sys::napi_get_typedarray_info(
          env,
          napi_val,
          &mut typed_array_type,
          &mut length,
          &mut data,
          &mut array_buffer,
          &mut byte_offset,
        )
      },
      "Get TypedArray info failed"
    )?;
    if typed_array_type != TypedArrayType::Uint8Clamped as i32 {
      return Err(Error::new(
        Status::InvalidArg,
        format!("Expected $name, got {}", typed_array_type),
      ));
    }
    length /= TypedArrayType::Uint8Clamped.byte_len();
    Ok(Self {
      inner: if length == 0 {
        &mut []
      } else {
        unsafe { core::slice::from_raw_parts_mut(data.cast(), length) }
      },
      raw_value: napi_val,
    })
  }
}

impl ToNapiValue for Uint8ClampedSlice<'_> {
  #[allow(unused_variables)]
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    Ok(val.raw_value)
  }
}

impl TypeName for Uint8ClampedSlice<'_> {
  fn type_name() -> &'static str {
    "Uint8ClampedArray"
  }

  fn value_type() -> ValueType {
    ValueType::Object
  }
}

impl ValidateNapiValue for Uint8ClampedSlice<'_> {
  unsafe fn validate(env: sys::napi_env, napi_val: sys::napi_value) -> Result<sys::napi_value> {
    let mut is_typedarray = false;
    check_status!(
      unsafe { sys::napi_is_typedarray(env, napi_val, &mut is_typedarray) },
      "Failed to validate typed buffer"
    )?;
    if !is_typedarray {
      return Err(Error::new(
        Status::InvalidArg,
        "Expected a TypedArray value".to_owned(),
      ));
    }
    Ok(ptr::null_mut())
  }
}

impl AsRef<[u8]> for Uint8ClampedSlice<'_> {
  fn as_ref(&self) -> &[u8] {
    self.inner
  }
}

impl Deref for Uint8ClampedSlice<'_> {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    self.inner
  }
}

impl DerefMut for Uint8ClampedSlice<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.inner
  }
}

impl<T: Into<Vec<u8>>> From<T> for Uint8Array {
  fn from(data: T) -> Self {
    Uint8Array::new(data.into())
  }
}

impl<T: Into<Vec<u8>>> From<T> for Uint8ClampedArray {
  fn from(data: T) -> Self {
    Uint8ClampedArray::new(data.into())
  }
}

impl<T: Into<Vec<u16>>> From<T> for Uint16Array {
  fn from(data: T) -> Self {
    Uint16Array::new(data.into())
  }
}

impl<T: Into<Vec<u32>>> From<T> for Uint32Array {
  fn from(data: T) -> Self {
    Uint32Array::new(data.into())
  }
}

impl<T: Into<Vec<i8>>> From<T> for Int8Array {
  fn from(data: T) -> Self {
    Int8Array::new(data.into())
  }
}

impl<T: Into<Vec<i16>>> From<T> for Int16Array {
  fn from(data: T) -> Self {
    Int16Array::new(data.into())
  }
}

impl<T: Into<Vec<i32>>> From<T> for Int32Array {
  fn from(data: T) -> Self {
    Int32Array::new(data.into())
  }
}

impl<T: Into<Vec<f32>>> From<T> for Float32Array {
  fn from(data: T) -> Self {
    Float32Array::new(data.into())
  }
}

impl<T: Into<Vec<f64>>> From<T> for Float64Array {
  fn from(data: T) -> Self {
    Float64Array::new(data.into())
  }
}

#[cfg(feature = "napi6")]
impl<T: Into<Vec<i64>>> From<T> for BigInt64Array {
  fn from(data: T) -> Self {
    BigInt64Array::new(data.into())
  }
}
#[cfg(feature = "napi6")]
impl<T: Into<Vec<u64>>> From<T> for BigUint64Array {
  fn from(data: T) -> Self {
    BigUint64Array::new(data.into())
  }
}
