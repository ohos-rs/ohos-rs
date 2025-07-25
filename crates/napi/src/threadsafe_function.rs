#![allow(clippy::single_component_path_imports)]

#[cfg(feature = "tokio_rt")]
use std::convert::identity;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr::{self, null_mut};
use std::sync::{
  self,
  atomic::{AtomicPtr, Ordering},
  Arc, RwLock, RwLockWriteGuard,
};

use crate::bindgen_runtime::{
  FromNapiValue, JsValuesTupleIntoVec, TypeName, Unknown, ValidateNapiValue,
};
use crate::{
  check_status, get_error_message_and_stack_trace, sys, Env, Error, JsError, Result, Status,
};

/// ThreadSafeFunction Context object
/// the `value` is the value passed to `call` method
pub struct ThreadsafeCallContext<T: 'static> {
  pub env: Env,
  pub value: T,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreadsafeFunctionCallMode {
  NonBlocking,
  Blocking,
}

impl From<ThreadsafeFunctionCallMode> for sys::napi_threadsafe_function_call_mode {
  fn from(value: ThreadsafeFunctionCallMode) -> Self {
    match value {
      ThreadsafeFunctionCallMode::Blocking => sys::ThreadsafeFunctionCallMode::blocking,
      ThreadsafeFunctionCallMode::NonBlocking => sys::ThreadsafeFunctionCallMode::nonblocking,
    }
  }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreadsafeFunctionPriority {
  Immediate,
  High,
  Low,
  Idle,
}

impl From<ThreadsafeFunctionPriority> for sys::napi_task_priority {
  fn from(value: ThreadsafeFunctionPriority) -> Self {
    match value {
      ThreadsafeFunctionPriority::Immediate => sys::napi_task_priority::napi_priority_immediate,
      ThreadsafeFunctionPriority::High => sys::napi_task_priority::napi_priority_high,
      ThreadsafeFunctionPriority::Low => sys::napi_task_priority::napi_priority_low,
      ThreadsafeFunctionPriority::Idle => sys::napi_task_priority::napi_priority_idle,
    }
  }
}

struct ThreadsafeFunctionHandle {
  raw: AtomicPtr<sys::napi_threadsafe_function__>,
  aborted: RwLock<bool>,
}

impl ThreadsafeFunctionHandle {
  /// create a Arc to hold the `ThreadsafeFunctionHandle`
  fn new(raw: sys::napi_threadsafe_function) -> Arc<Self> {
    Arc::new(Self {
      raw: AtomicPtr::new(raw),
      aborted: RwLock::new(false),
    })
  }

  /// Lock `aborted` with read access, call `f` with the value of `aborted`, then unlock it
  fn with_read_aborted<RT, F>(&self, f: F) -> RT
  where
    F: FnOnce(bool) -> RT,
  {
    let aborted_guard = self
      .aborted
      .read()
      .expect("Threadsafe Function aborted lock failed");
    f(*aborted_guard)
  }

  /// Lock `aborted` with write access, call `f` with the `RwLockWriteGuard`, then unlock it
  fn with_write_aborted<RT, F>(&self, f: F) -> RT
  where
    F: FnOnce(RwLockWriteGuard<bool>) -> RT,
  {
    let aborted_guard = self
      .aborted
      .write()
      .expect("Threadsafe Function aborted lock failed");
    f(aborted_guard)
  }

  #[allow(clippy::arc_with_non_send_sync)]
  fn null() -> Arc<Self> {
    Self::new(null_mut())
  }

  fn get_raw(&self) -> sys::napi_threadsafe_function {
    self.raw.load(Ordering::SeqCst)
  }

  fn set_raw(&self, raw: sys::napi_threadsafe_function) {
    self.raw.store(raw, Ordering::SeqCst)
  }
}

impl Drop for ThreadsafeFunctionHandle {
  fn drop(&mut self) {
    self.with_read_aborted(|aborted| {
      if !aborted {
        let raw = self.get_raw();
        // if ThreadsafeFunction::create failed, the raw will be null and we don't need to release it
        if !raw.is_null() {
          let release_status = unsafe {
            sys::napi_release_threadsafe_function(
              self.get_raw(),
              sys::ThreadsafeFunctionReleaseMode::release,
            )
          };
          assert!(
            release_status == sys::Status::napi_ok,
            "Threadsafe Function release failed {}",
            Status::from(release_status)
          );
        }
      }
    })
  }
}

#[repr(u8)]
enum ThreadsafeFunctionCallVariant {
  Direct,
  WithCallback,
}

struct ThreadsafeFunctionCallJsBackData<T, Return = Unknown<'static>> {
  data: T,
  call_variant: ThreadsafeFunctionCallVariant,
  callback: Box<dyn FnOnce(Result<Return>, Env) -> Result<()>>,
}

/// Communicate with the addon's main thread by invoking a JavaScript function from other threads.
///
/// ## Example
/// An example of using `ThreadsafeFunction`:
///
/// ```rust
/// use std::thread;
/// use std::sync::Arc;
///
/// use napi_ohos::{
///     threadsafe_function::{
///         ThreadSafeCallContext, ThreadsafeFunctionCallMode, ThreadsafeFunctionReleaseMode,
///     },
/// };
/// use napi_derive_ohos::napi;
///
/// #[napi]
/// pub fn call_threadsafe_function(callback: Arc<ThreadsafeFunction<(u32, bool, String), ()>>) {
///   let tsfn_cloned = tsfn.clone();
///
///   thread::spawn(move || {
///       let output: Vec<u32> = vec![0, 1, 2, 3];
///       // It's okay to call a threadsafe function multiple times.
///       tsfn.call(Ok((1, false, "NAPI-RS".into())), ThreadsafeFunctionCallMode::Blocking);
///       tsfn.call(Ok((2, true, "NAPI-RS".into())), ThreadsafeFunctionCallMode::NonBlocking);
///   });
///
///   thread::spawn(move || {
///       tsfn_cloned.call((3, false, "NAPI-RS".into())), ThreadsafeFunctionCallMode::NonBlocking);
///   });
/// }
/// ```
pub struct ThreadsafeFunction<
  T: 'static,
  Return: 'static + FromNapiValue = Unknown<'static>,
  CallJsBackArgs: 'static + JsValuesTupleIntoVec = T,
  ErrorStatus: AsRef<str> + From<Status> = Status,
  const CalleeHandled: bool = true,
  const Weak: bool = false,
  const MaxQueueSize: usize = 0,
> {
  handle: Arc<ThreadsafeFunctionHandle>,
  _phantom: PhantomData<(T, CallJsBackArgs, Return, ErrorStatus)>,
}

unsafe impl<
    T: 'static,
    Return: FromNapiValue,
    CallJsBackArgs: 'static + JsValuesTupleIntoVec,
    ErrorStatus: AsRef<str> + From<Status>,
    const CalleeHandled: bool,
    const Weak: bool,
    const MaxQueueSize: usize,
  > Send
  for ThreadsafeFunction<
    T,
    Return,
    CallJsBackArgs,
    ErrorStatus,
    { CalleeHandled },
    { Weak },
    { MaxQueueSize },
  >
{
}

unsafe impl<
    T: 'static,
    Return: FromNapiValue,
    CallJsBackArgs: 'static + JsValuesTupleIntoVec,
    ErrorStatus: AsRef<str> + From<Status>,
    const CalleeHandled: bool,
    const Weak: bool,
    const MaxQueueSize: usize,
  > Sync
  for ThreadsafeFunction<
    T,
    Return,
    CallJsBackArgs,
    ErrorStatus,
    { CalleeHandled },
    { Weak },
    { MaxQueueSize },
  >
{
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    Return: FromNapiValue,
    ErrorStatus: AsRef<str> + From<Status>,
    const CalleeHandled: bool,
    const Weak: bool,
    const MaxQueueSize: usize,
  > FromNapiValue
  for ThreadsafeFunction<T, Return, T, ErrorStatus, { CalleeHandled }, { Weak }, { MaxQueueSize }>
{
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    Self::create(env, napi_val, |ctx| Ok(ctx.value))
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    Return: FromNapiValue,
    ErrorStatus: AsRef<str> + From<Status>,
    const CalleeHandled: bool,
    const Weak: bool,
    const MaxQueueSize: usize,
  > TypeName
  for ThreadsafeFunction<T, Return, T, ErrorStatus, { CalleeHandled }, { Weak }, { MaxQueueSize }>
{
  fn type_name() -> &'static str {
    "ThreadsafeFunction"
  }

  fn value_type() -> crate::ValueType {
    crate::ValueType::Function
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    Return: FromNapiValue,
    ErrorStatus: AsRef<str> + From<Status>,
    const CalleeHandled: bool,
    const Weak: bool,
    const MaxQueueSize: usize,
  > ValidateNapiValue
  for ThreadsafeFunction<T, Return, T, ErrorStatus, { CalleeHandled }, { Weak }, { MaxQueueSize }>
{
}

impl<
    T: 'static,
    Return: FromNapiValue,
    CallJsBackArgs: 'static + JsValuesTupleIntoVec,
    ErrorStatus: AsRef<str> + From<Status>,
    const CalleeHandled: bool,
    const Weak: bool,
    const MaxQueueSize: usize,
  >
  ThreadsafeFunction<
    T,
    Return,
    CallJsBackArgs,
    ErrorStatus,
    { CalleeHandled },
    { Weak },
    { MaxQueueSize },
  >
{
  // See [napi_create_threadsafe_function](https://nodejs.org/api/n-api.html#n_api_napi_create_threadsafe_function)
  // for more information.
  pub(crate) fn create<
    NewArgs: 'static + JsValuesTupleIntoVec,
    R: 'static + FnMut(ThreadsafeCallContext<T>) -> Result<NewArgs>,
  >(
    env: sys::napi_env,
    func: sys::napi_value,
    callback: R,
  ) -> Result<
    ThreadsafeFunction<
      T,
      Return,
      NewArgs,
      ErrorStatus,
      { CalleeHandled },
      { Weak },
      { MaxQueueSize },
    >,
  > {
    let mut async_resource_name = ptr::null_mut();
    static THREAD_SAFE_FUNCTION_ASYNC_RESOURCE_NAME: &str = "napi_rs_threadsafe_function";

    #[cfg(feature = "napi10")]
    {
      let mut copied = false;
      check_status!(
        unsafe {
          sys::node_api_create_external_string_latin1(
            env,
            THREAD_SAFE_FUNCTION_ASYNC_RESOURCE_NAME.as_ptr().cast(),
            27,
            None,
            ptr::null_mut(),
            &mut async_resource_name,
            &mut copied,
          )
        },
        "Create external string latin1 in ThreadsafeFunction::create failed"
      )?;
    }

    #[cfg(not(feature = "napi10"))]
    {
      check_status!(
        unsafe {
          sys::napi_create_string_utf8(
            env,
            THREAD_SAFE_FUNCTION_ASYNC_RESOURCE_NAME.as_ptr().cast(),
            27,
            &mut async_resource_name,
          )
        },
        "Create string utf8 in ThreadsafeFunction::create failed"
      )?;
    }

    let mut raw_tsfn = ptr::null_mut();
    let callback_ptr = Box::into_raw(Box::new(callback));
    let handle = ThreadsafeFunctionHandle::null();
    check_status!(
      unsafe {
        sys::napi_create_threadsafe_function(
          env,
          func,
          ptr::null_mut(),
          async_resource_name,
          MaxQueueSize,
          1,
          Arc::downgrade(&handle).into_raw().cast_mut().cast(), // pass handler to thread_finalize_cb
          Some(thread_finalize_cb::<T, NewArgs, R>),
          callback_ptr.cast(),
          Some(call_js_cb::<T, Return, NewArgs, ErrorStatus, R, CalleeHandled>),
          &mut raw_tsfn,
        )
      },
      "Create threadsafe function in ThreadsafeFunction::create failed"
    )?;
    handle.set_raw(raw_tsfn);

    // Weak ThreadsafeFunction will not prevent the event loop from exiting
    if Weak {
      check_status!(
        unsafe { sys::napi_unref_threadsafe_function(env, raw_tsfn) },
        "Unref threadsafe function failed in Weak mode"
      )?;
    }

    Ok(ThreadsafeFunction {
      handle,
      _phantom: PhantomData,
    })
  }

  pub fn aborted(&self) -> bool {
    self.handle.with_read_aborted(|aborted| aborted)
  }

  /// Get the raw `ThreadSafeFunction` pointer
  pub fn raw(&self) -> sys::napi_threadsafe_function {
    self.handle.get_raw()
  }
}

impl<
    T: 'static,
    Return: FromNapiValue,
    CallJsBackArgs: 'static + JsValuesTupleIntoVec,
    ErrorStatus: AsRef<str> + From<Status>,
    const Weak: bool,
    const MaxQueueSize: usize,
  > ThreadsafeFunction<T, Return, CallJsBackArgs, ErrorStatus, true, { Weak }, { MaxQueueSize }>
{
  /// See [napi_call_threadsafe_function](https://nodejs.org/api/n-api.html#n_api_napi_call_threadsafe_function)
  /// for more information.
  pub fn call(&self, value: Result<T, ErrorStatus>, mode: ThreadsafeFunctionCallMode) -> Status {
    self.handle.with_read_aborted(|aborted| {
      if aborted {
        return Status::Closing;
      }

      unsafe {
        sys::napi_call_threadsafe_function(
          self.handle.get_raw(),
          Box::into_raw(Box::new(value.map(|data| {
            ThreadsafeFunctionCallJsBackData {
              data,
              call_variant: ThreadsafeFunctionCallVariant::Direct,
              callback: Box::new(|_d: Result<Return>, _| Ok(())),
            }
          })))
          .cast(),
          mode.into(),
        )
      }
      .into()
    })
  }

  /// Call the ThreadsafeFunction, and handle the return value with a callback
  pub fn call_with_return_value<F: 'static + FnOnce(Result<Return>, Env) -> Result<()>>(
    &self,
    value: Result<T, ErrorStatus>,
    mode: ThreadsafeFunctionCallMode,
    cb: F,
  ) -> Status {
    self.handle.with_read_aborted(|aborted| {
      if aborted {
        return Status::Closing;
      }

      unsafe {
        sys::napi_call_threadsafe_function(
          self.handle.get_raw(),
          Box::into_raw(Box::new(value.map(|data| {
            ThreadsafeFunctionCallJsBackData {
              data,
              call_variant: ThreadsafeFunctionCallVariant::WithCallback,
              callback: Box::new(move |d: Result<Return>, env: Env| cb(d, env)),
            }
          })))
          .cast(),
          mode.into(),
        )
      }
      .into()
    })
  }

  #[cfg(feature = "tokio_rt")]
  /// Call the ThreadsafeFunction, and handle the return value with in `async` way
  pub async fn call_async(&self, value: Result<T, ErrorStatus>) -> Result<Return> {
    let (sender, receiver) = tokio::sync::oneshot::channel::<Result<Return>>();

    self.handle.with_read_aborted(|aborted| {
      if aborted {
        return Err(crate::Error::from_status(Status::Closing));
      }

      check_status!(
        unsafe {
          sys::napi_call_threadsafe_function(
            self.handle.get_raw(),
            Box::into_raw(Box::new(value.map(|data| {
              ThreadsafeFunctionCallJsBackData {
                data,
                call_variant: ThreadsafeFunctionCallVariant::WithCallback,
                callback: Box::new(move |d: Result<Return>, _| {
                  sender
                    .send(d)
                    // The only reason for send to return Err is if the receiver isn't listening
                    // Not hiding the error would result in a napi_fatal_error call, it's safe to ignore it instead.
                    .or(Ok(()))
                }),
              }
            })))
            .cast(),
            ThreadsafeFunctionCallMode::NonBlocking.into(),
          )
        },
        "Threadsafe function call_async failed"
      )
    })?;
    receiver
      .await
      .map_err(|_| {
        crate::Error::new(
          Status::GenericFailure,
          "Receive value from threadsafe function sender failed",
        )
      })
      .and_then(identity)
  }
}

impl<
    T: 'static,
    Return: FromNapiValue,
    CallJsBackArgs: 'static + JsValuesTupleIntoVec,
    ErrorStatus: AsRef<str> + From<Status>,
    const Weak: bool,
    const MaxQueueSize: usize,
  > ThreadsafeFunction<T, Return, CallJsBackArgs, ErrorStatus, false, { Weak }, { MaxQueueSize }>
{
  /// See [napi_call_threadsafe_function](https://nodejs.org/api/n-api.html#n_api_napi_call_threadsafe_function)
  /// for more information.
  pub fn call(&self, value: T, mode: ThreadsafeFunctionCallMode) -> Status {
    self.handle.with_read_aborted(|aborted| {
      if aborted {
        return Status::Closing;
      }

      unsafe {
        sys::napi_call_threadsafe_function(
          self.handle.get_raw(),
          Box::into_raw(Box::new(ThreadsafeFunctionCallJsBackData {
            data: value,
            call_variant: ThreadsafeFunctionCallVariant::Direct,
            callback: Box::new(|_d: Result<Return>, _: Env| Ok(())),
          }))
          .cast(),
          mode.into(),
        )
      }
      .into()
    })
  }

  /// Call the ThreadsafeFunction with priority, insert it at the end of queue.
  #[cfg(target_env = "ohos")]
  pub fn call_with_priority(&self, value: T, priority: ThreadsafeFunctionPriority) -> Status {
    self.handle.with_read_aborted(|aborted| {
      if aborted {
        return Status::Closing;
      }

      unsafe {
        sys::napi_call_threadsafe_function_with_priority(
          self.handle.get_raw(),
          Box::into_raw(Box::new(ThreadsafeFunctionCallJsBackData {
            data: value,
            call_variant: ThreadsafeFunctionCallVariant::Direct,
            callback: Box::new(|_d: Result<Return>, _: Env| Ok(())),
          }))
          .cast(),
          priority.into(),
          true,
        )
      }
      .into()
    })
  }

  /// Call the ThreadsafeFunction with priority, insert it at the start of queue.
  #[cfg(target_env = "ohos")]
  pub fn call_with_priority_first(&self, value: T, priority: ThreadsafeFunctionPriority) -> Status {
    self.handle.with_read_aborted(|aborted| {
      if aborted {
        return Status::Closing;
      }

      unsafe {
        sys::napi_call_threadsafe_function_with_priority(
          self.handle.get_raw(),
          Box::into_raw(Box::new(ThreadsafeFunctionCallJsBackData {
            data: value,
            call_variant: ThreadsafeFunctionCallVariant::Direct,
            callback: Box::new(|_d: Result<Return>, _: Env| Ok(())),
          }))
          .cast(),
          priority.into(),
          false,
        )
      }
      .into()
    })
  }

  /// Call the ThreadsafeFunction, and handle the return value with a callback
  pub fn call_with_return_value<F: 'static + FnOnce(Result<Return>, Env) -> Result<()>>(
    &self,
    value: T,
    mode: ThreadsafeFunctionCallMode,
    cb: F,
  ) -> Status {
    self.handle.with_read_aborted(|aborted| {
      if aborted {
        return Status::Closing;
      }

      unsafe {
        sys::napi_call_threadsafe_function(
          self.handle.get_raw(),
          Box::into_raw(Box::new(ThreadsafeFunctionCallJsBackData {
            data: value,
            call_variant: ThreadsafeFunctionCallVariant::WithCallback,
            callback: Box::new(cb),
          }))
          .cast(),
          mode.into(),
        )
      }
      .into()
    })
  }

  #[cfg(feature = "tokio_rt")]
  /// Call the ThreadsafeFunction, and handle the return value with in `async` way
  pub async fn call_async(&self, value: T) -> Result<Return> {
    let (sender, receiver) = tokio::sync::oneshot::channel::<Return>();

    self.handle.with_read_aborted(|aborted| {
      if aborted {
        return Err(crate::Error::from_status(Status::Closing));
      }

      check_status!(unsafe {
        sys::napi_call_threadsafe_function(
          self.handle.get_raw(),
          Box::into_raw(Box::new(ThreadsafeFunctionCallJsBackData {
            data: value,
            call_variant: ThreadsafeFunctionCallVariant::WithCallback,
            callback: Box::new(move |d, _| {
              d.and_then(|d| {
                sender
                  .send(d)
                  // The only reason for send to return Err is if the receiver isn't listening
                  // Not hiding the error would result in a napi_fatal_error call, it's safe to ignore it instead.
                  .or(Ok(()))
              })
            }),
          }))
          .cast(),
          ThreadsafeFunctionCallMode::NonBlocking.into(),
        )
      })
    })?;

    receiver
      .await
      .map_err(|err| crate::Error::new(Status::GenericFailure, format!("{err}")))
  }
}

unsafe extern "C" fn thread_finalize_cb<T: 'static, V: 'static + JsValuesTupleIntoVec, R>(
  #[allow(unused_variables)] env: sys::napi_env,
  finalize_data: *mut c_void,
  finalize_hint: *mut c_void,
) where
  R: 'static + FnMut(ThreadsafeCallContext<T>) -> Result<V>,
{
  let handle_option: Option<Arc<ThreadsafeFunctionHandle>> =
    unsafe { sync::Weak::from_raw(finalize_data.cast()).upgrade() };

  if let Some(handle) = handle_option {
    handle.with_write_aborted(|mut aborted_guard| {
      if !*aborted_guard {
        *aborted_guard = true;
      }
    });
  }

  // cleanup
  drop(unsafe { Box::<R>::from_raw(finalize_hint.cast()) });
}

unsafe extern "C" fn call_js_cb<
  T: 'static,
  Return: FromNapiValue,
  V: 'static + JsValuesTupleIntoVec,
  ErrorStatus: AsRef<str> + From<Status>,
  R,
  const CalleeHandled: bool,
>(
  raw_env: sys::napi_env,
  js_callback: sys::napi_value,
  context: *mut c_void,
  data: *mut c_void,
) where
  R: 'static + FnMut(ThreadsafeCallContext<T>) -> Result<V>,
{
  // env and/or callback can be null when shutting down
  if raw_env.is_null() || js_callback.is_null() {
    return;
  }

  let callback: &mut R = unsafe { Box::leak(Box::from_raw(context.cast())) };
  let val = unsafe {
    if CalleeHandled {
      *Box::<Result<ThreadsafeFunctionCallJsBackData<T, Return>, ErrorStatus>>::from_raw(
        data.cast(),
      )
    } else {
      Ok(*Box::<ThreadsafeFunctionCallJsBackData<T, Return>>::from_raw(data.cast()))
    }
  };

  let mut recv = ptr::null_mut();
  unsafe { sys::napi_get_undefined(raw_env, &mut recv) };

  let ret = val.and_then(|v| {
    (callback)(ThreadsafeCallContext {
      env: Env::from_raw(raw_env),
      value: v.data,
    })
    .and_then(|ret| Ok((ret.into_vec(raw_env)?, v.call_variant, v.callback)))
    .map_err(|err| Error::new(err.status.into(), err.reason.clone()))
  });

  // Follow async callback conventions: https://nodejs.org/en/knowledge/errors/what-are-the-error-conventions/
  // Check if the Result is okay, if so, pass a null as the first (error) argument automatically.
  // If the Result is an error, pass that as the first argument.
  let status = match ret {
    Ok((values, call_variant, callback)) => {
      let args: Vec<sys::napi_value> = if CalleeHandled {
        let mut js_null = ptr::null_mut();
        unsafe { sys::napi_get_null(raw_env, &mut js_null) };
        core::iter::once(js_null).chain(values).collect()
      } else {
        values
      };
      let mut return_value = ptr::null_mut();
      let mut status = sys::napi_call_function(
        raw_env,
        recv,
        js_callback,
        args.len(),
        args.as_ptr(),
        &mut return_value,
      );
      if let ThreadsafeFunctionCallVariant::WithCallback = call_variant {
        // throw Error in JavaScript callback
        let callback_arg = if status == sys::Status::napi_pending_exception {
          let mut exception = ptr::null_mut();
          unsafe { sys::napi_get_and_clear_last_exception(raw_env, &mut exception) };
          let mut error_reference = ptr::null_mut();
          let raw_status = status;
          status =
            unsafe { sys::napi_create_reference(raw_env, exception, 1, &mut error_reference) };

          get_error_message_and_stack_trace(raw_env, exception).and_then(|reason| {
            Err(Error {
              maybe_raw: error_reference,
              maybe_env: raw_env,
              status: Status::from(raw_status),
              reason,
            })
          })
        } else {
          unsafe { Return::from_napi_value(raw_env, return_value) }
        };
        if let Err(err) = callback(callback_arg, Env::from_raw(raw_env)) {
          unsafe { sys::napi_fatal_exception(raw_env, JsError::from(err).into_value(raw_env)) };
        }
      }
      status
    }
    Err(e) if !CalleeHandled => unsafe {
      sys::napi_fatal_exception(raw_env, JsError::from(e).into_value(raw_env))
    },
    Err(e) => unsafe {
      sys::napi_call_function(
        raw_env,
        recv,
        js_callback,
        1,
        [JsError::from(e).into_value(raw_env)].as_mut_ptr(),
        ptr::null_mut(),
      )
    },
  };
  handle_call_js_cb_status(status, raw_env)
}

fn handle_call_js_cb_status(status: sys::napi_status, raw_env: sys::napi_env) {
  if status == sys::Status::napi_ok {
    return;
  }
  if status == sys::Status::napi_pending_exception {
    let mut error_result = ptr::null_mut();
    assert_eq!(
      unsafe { sys::napi_get_and_clear_last_exception(raw_env, &mut error_result) },
      sys::Status::napi_ok
    );

    // When shutting down, napi_fatal_exception sometimes returns another exception
    let stat = unsafe { sys::napi_fatal_exception(raw_env, error_result) };
    assert!(stat == sys::Status::napi_ok || stat == sys::Status::napi_pending_exception);
  } else {
    let error_code: Status = status.into();
    let mut error_code_value = ptr::null_mut();
    assert_eq!(
      unsafe {
        sys::napi_create_string_utf8(
          raw_env,
          error_code.as_ref().as_ptr().cast(),
          error_code.as_ref().len() as isize,
          &mut error_code_value,
        )
      },
      sys::Status::napi_ok,
    );
    const ERROR_MSG: &str = "Call JavaScript callback failed in threadsafe function";
    let mut error_msg_value = ptr::null_mut();
    assert_eq!(
      unsafe {
        sys::napi_create_string_utf8(
          raw_env,
          ERROR_MSG.as_ptr().cast(),
          ERROR_MSG.len() as isize,
          &mut error_msg_value,
        )
      },
      sys::Status::napi_ok,
    );
    let mut error_value = ptr::null_mut();
    assert_eq!(
      unsafe {
        sys::napi_create_error(raw_env, error_code_value, error_msg_value, &mut error_value)
      },
      sys::Status::napi_ok,
    );
    assert_eq!(
      unsafe { sys::napi_fatal_exception(raw_env, error_value) },
      sys::Status::napi_ok
    );
  }
}

/// This is a placeholder type that is used to indicate that the return value of a threadsafe function is unknown.
/// Use this type when you don't care about the return value of a threadsafe function.
///
/// And you can't get the value of it as well because it's just a placeholder.
pub struct UnknownReturnValue;

impl TypeName for UnknownReturnValue {
  fn type_name() -> &'static str {
    "UnknownReturnValue"
  }

  fn value_type() -> crate::ValueType {
    crate::ValueType::Unknown
  }
}

impl ValidateNapiValue for UnknownReturnValue {}

impl FromNapiValue for UnknownReturnValue {
  unsafe fn from_napi_value(_env: sys::napi_env, _napi_val: sys::napi_value) -> Result<Self> {
    Ok(UnknownReturnValue)
  }
}
