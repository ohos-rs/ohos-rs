#![allow(deprecated)]

use std::ptr;

use super::{Either, FromNapiValue, ToNapiValue, TypeName, Unknown, ValidateNapiValue};

#[cfg(feature = "napi4")]
use crate::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction};
use crate::{
  check_pending_exception, check_status, sys, Env, JsUndefined, NapiRaw, NapiValue, Result,
  ValueType,
};

pub trait JsValuesTupleIntoVec {
  fn into_vec(self, env: sys::napi_env) -> Result<Vec<sys::napi_value>>;
}

impl<T> JsValuesTupleIntoVec for T
where
  T: ToNapiValue,
{
  #[allow(clippy::not_unsafe_ptr_arg_deref)]
  fn into_vec(self, env: sys::napi_env) -> Result<Vec<sys::napi_value>> {
    // allow call function with `()` and function's arguments should be empty array
    if std::mem::size_of::<T>() == 0 {
      Ok(vec![])
    } else {
      Ok(vec![unsafe {
        <T as ToNapiValue>::to_napi_value(env, self)?
      }])
    }
  }
}
pub trait TupleFromSliceValues {
  #[allow(clippy::missing_safety_doc)]
  unsafe fn from_slice_values(env: sys::napi_env, values: &[sys::napi_value]) -> Result<Self>
  where
    Self: Sized;
}

#[repr(C)]
pub struct FnArgs<T> {
  pub data: T,
}

impl<T> From<T> for FnArgs<T> {
  fn from(value: T) -> Self {
    FnArgs { data: value }
  }
}

macro_rules! impl_tuple_conversion {
  ($($ident:ident),*) => {
    impl<$($ident: ToNapiValue),*> JsValuesTupleIntoVec for FnArgs<($($ident,)*)> {
      #[allow(clippy::not_unsafe_ptr_arg_deref)]
      fn into_vec(self, env: sys::napi_env) -> Result<Vec<sys::napi_value>> {
        #[allow(non_snake_case)]
        let ($($ident,)*) = self.data;
        Ok(vec![$(unsafe { <$ident as ToNapiValue>::to_napi_value(env, $ident)? }),*])
      }
    }

    impl<$($ident: FromNapiValue),*> TupleFromSliceValues for ($($ident,)*) {
      unsafe fn from_slice_values(env: sys::napi_env, values: &[sys::napi_value]) -> $crate::Result<Self> {
        #[allow(non_snake_case)]
        let [$($ident),*] = values.try_into().map_err(|_| crate::Error::new(
          crate::Status::InvalidArg,
          "Invalid number of arguments",
        ))?;
        Ok(($(
          unsafe { $ident::from_napi_value(env, $ident)?}
        ,)*))
      }
    }
  };
}

impl_tuple_conversion!(A);
impl_tuple_conversion!(A, B);
impl_tuple_conversion!(A, B, C);
impl_tuple_conversion!(A, B, C, D);
impl_tuple_conversion!(A, B, C, D, E);
impl_tuple_conversion!(A, B, C, D, E, F);
impl_tuple_conversion!(A, B, C, D, E, F, G);
impl_tuple_conversion!(A, B, C, D, E, F, G, H);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_tuple_conversion!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_tuple_conversion!(
  A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);

/// A JavaScript function.
/// It can only live in the scope of a function call.
/// If you want to use it outside the scope of a function call, you can turn it into a reference.
/// By calling the `create_ref` method.
pub struct Function<'scope, Args: JsValuesTupleIntoVec = Unknown, Return = Unknown> {
  pub(crate) env: sys::napi_env,
  pub(crate) value: sys::napi_value,
  pub(crate) _args: std::marker::PhantomData<Args>,
  pub(crate) _return: std::marker::PhantomData<Return>,
  _scope: std::marker::PhantomData<&'scope ()>,
}

impl<Args: JsValuesTupleIntoVec, Return> TypeName for Function<'_, Args, Return> {
  fn type_name() -> &'static str {
    "Function"
  }

  fn value_type() -> crate::ValueType {
    ValueType::Function
  }
}

impl<Args: JsValuesTupleIntoVec, Return> NapiRaw for Function<'_, Args, Return> {
  unsafe fn raw(&self) -> sys::napi_value {
    self.value
  }
}

impl<Args: JsValuesTupleIntoVec, Return> FromNapiValue for Function<'_, Args, Return> {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> Result<Self> {
    Ok(Function {
      env,
      value,
      _args: std::marker::PhantomData,
      _return: std::marker::PhantomData,
      _scope: std::marker::PhantomData,
    })
  }
}

impl<Args: JsValuesTupleIntoVec, Return> ValidateNapiValue for Function<'_, Args, Return> {}

impl<Args: JsValuesTupleIntoVec, Return> Function<'_, Args, Return> {
  /// Get the name of the JavaScript function.
  pub fn name(&self) -> Result<String> {
    let mut name = ptr::null_mut();
    check_status!(
      unsafe {
        sys::napi_get_named_property(self.env, self.value, c"name".as_ptr().cast(), &mut name)
      },
      "Get function name failed"
    )?;
    unsafe { String::from_napi_value(self.env, name) }
  }

  /// Create a reference to the JavaScript function.
  pub fn create_ref(&self) -> Result<FunctionRef<Args, Return>> {
    let mut reference = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_create_reference(self.env, self.value, 1, &mut reference) },
      "Create reference failed"
    )?;
    Ok(FunctionRef {
      inner: reference,
      env: self.env,
      _args: std::marker::PhantomData,
      _return: std::marker::PhantomData,
    })
  }

  /// Create a new instance of the JavaScript Class.
  pub fn new_instance(&self, args: Args) -> Result<Unknown> {
    let mut raw_instance = ptr::null_mut();
    let mut args = args.into_vec(self.env)?;
    check_status!(
      unsafe {
        sys::napi_new_instance(
          self.env,
          self.value,
          args.len(),
          args.as_mut_ptr().cast(),
          &mut raw_instance,
        )
      },
      "Create new instance failed"
    )?;
    unsafe { Unknown::from_napi_value(self.env, raw_instance) }
  }

  #[cfg(feature = "napi4")]
  /// Create a threadsafe function from the JavaScript function.
  pub fn build_threadsafe_function<T: 'static>(
    &self,
  ) -> ThreadsafeFunctionBuilder<T, Args, Return> {
    ThreadsafeFunctionBuilder {
      env: self.env,
      value: self.value,
      _args: std::marker::PhantomData,
      _return: std::marker::PhantomData,
    }
  }
}

impl<Args: JsValuesTupleIntoVec, Return: FromNapiValue> Function<'_, Args, Return> {
  /// Call the JavaScript function.
  /// `this` in the JavaScript function will be `undefined`.
  /// If you want to specify `this`, you can use the `apply` method.
  pub fn call(&self, args: Args) -> Result<Return> {
    let mut raw_this = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_undefined(self.env, &mut raw_this) },
      "Get undefined value failed"
    )?;
    let args_ptr = args.into_vec(self.env)?;
    let mut raw_return = ptr::null_mut();
    check_pending_exception!(
      self.env,
      unsafe {
        sys::napi_call_function(
          self.env,
          raw_this,
          self.value,
          args_ptr.len(),
          args_ptr.as_ptr(),
          &mut raw_return,
        )
      },
      "Call Function failed"
    )?;
    unsafe { Return::from_napi_value(self.env, raw_return) }
  }

  /// Call the JavaScript function.
  /// `this` in the JavaScript function will be the provided `this`.
  pub fn apply<Context: ToNapiValue>(&self, this: Context, args: Args) -> Result<Return> {
    let raw_this = unsafe { Context::to_napi_value(self.env, this) }?;
    let args_ptr = args.into_vec(self.env)?;
    let mut raw_return = ptr::null_mut();
    check_status!(
      unsafe {
        sys::napi_call_function(
          self.env,
          raw_this,
          self.value,
          args_ptr.len(),
          args_ptr.as_ptr(),
          &mut raw_return,
        )
      },
      "Call Function failed"
    )?;
    unsafe { Return::from_napi_value(self.env, raw_return) }
  }

  /// Call `Function.bind`
  pub fn bind<T: ToNapiValue>(&self, this: T) -> Result<Function<'_, Args, Return>> {
    let raw_this = unsafe { T::to_napi_value(self.env, this) }?;
    let mut bind_function = ptr::null_mut();
    check_status!(
      unsafe {
        sys::napi_get_named_property(self.env, self.value, c"bind".as_ptr(), &mut bind_function)
      },
      "Get bind function failed"
    )?;
    let mut bound_function = ptr::null_mut();
    check_status!(
      unsafe {
        sys::napi_call_function(
          self.env,
          self.value,
          bind_function,
          1,
          [raw_this].as_ptr(),
          &mut bound_function,
        )
      },
      "Bind function failed"
    )?;
    Ok(Function {
      env: self.env,
      value: bound_function,
      _args: std::marker::PhantomData,
      _return: std::marker::PhantomData,
      _scope: std::marker::PhantomData,
    })
  }
}

#[cfg(feature = "napi4")]
pub struct ThreadsafeFunctionBuilder<
  'env,
  T: 'static,
  Args: 'static + JsValuesTupleIntoVec,
  Return,
  const CalleeHandled: bool = false,
  const Weak: bool = false,
  const MaxQueueSize: usize = 0,
> {
  pub(crate) env: sys::napi_env,
  pub(crate) value: sys::napi_value,
  _args: std::marker::PhantomData<(T, &'env Args)>,
  _return: std::marker::PhantomData<Return>,
}

#[cfg(feature = "napi4")]
impl<
    'env,
    T: 'static,
    Args: 'static + JsValuesTupleIntoVec,
    Return: FromNapiValue,
    const CalleeHandled: bool,
    const Weak: bool,
    const MaxQueueSize: usize,
  > ThreadsafeFunctionBuilder<'env, T, Args, Return, CalleeHandled, Weak, MaxQueueSize>
{
  pub fn weak<const NewWeak: bool>(
    self,
  ) -> ThreadsafeFunctionBuilder<'env, T, Args, Return, CalleeHandled, NewWeak, MaxQueueSize> {
    ThreadsafeFunctionBuilder {
      env: self.env,
      value: self.value,
      _args: std::marker::PhantomData,
      _return: std::marker::PhantomData,
    }
  }

  pub fn callee_handled<const NewCalleeHandled: bool>(
    self,
  ) -> ThreadsafeFunctionBuilder<'env, T, Args, Return, NewCalleeHandled, Weak, MaxQueueSize> {
    ThreadsafeFunctionBuilder {
      env: self.env,
      value: self.value,
      _args: std::marker::PhantomData,
      _return: std::marker::PhantomData,
    }
  }

  pub fn max_queue_size<const NewMaxQueueSize: usize>(
    self,
  ) -> ThreadsafeFunctionBuilder<'env, T, Args, Return, CalleeHandled, Weak, NewMaxQueueSize> {
    ThreadsafeFunctionBuilder {
      env: self.env,
      value: self.value,
      _args: std::marker::PhantomData,
      _return: std::marker::PhantomData,
    }
  }

  pub fn build_callback<CallJsBackArgs, Callback>(
    &self,
    call_js_back: Callback,
  ) -> Result<ThreadsafeFunction<T, Return, CallJsBackArgs, CalleeHandled, Weak, MaxQueueSize>>
  where
    CallJsBackArgs: 'static + JsValuesTupleIntoVec,
    Callback: 'static + Send + FnMut(ThreadsafeCallContext<T>) -> Result<CallJsBackArgs>,
  {
    ThreadsafeFunction::<T, Return, Args, CalleeHandled, Weak, MaxQueueSize>::create(
      self.env,
      self.value,
      call_js_back,
    )
  }
}

#[cfg(feature = "napi4")]
impl<
    T: 'static + JsValuesTupleIntoVec,
    Return: FromNapiValue,
    const CalleeHandled: bool,
    const Weak: bool,
    const MaxQueueSize: usize,
  > ThreadsafeFunctionBuilder<'_, T, T, Return, CalleeHandled, Weak, MaxQueueSize>
{
  pub fn build(
    &self,
  ) -> Result<ThreadsafeFunction<T, Return, T, CalleeHandled, Weak, MaxQueueSize>> {
    unsafe { ThreadsafeFunction::from_napi_value(self.env, self.value) }
  }
}

/// A reference to a JavaScript function.
/// It can be used to outlive the scope of the function.
pub struct FunctionRef<Args: JsValuesTupleIntoVec, Return> {
  pub(crate) inner: sys::napi_ref,
  pub(crate) env: sys::napi_env,
  _args: std::marker::PhantomData<Args>,
  _return: std::marker::PhantomData<Return>,
}

unsafe impl<Args: JsValuesTupleIntoVec, Return> Sync for FunctionRef<Args, Return> {}

impl<Args: JsValuesTupleIntoVec, Return> FunctionRef<Args, Return> {
  pub fn borrow_back<'scope>(&self, env: &'scope Env) -> Result<Function<'scope, Args, Return>> {
    let mut value = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env.0, self.inner, &mut value) },
      "Get reference value failed"
    )?;
    Ok(Function {
      env: env.0,
      value,
      _args: std::marker::PhantomData,
      _return: std::marker::PhantomData,
      _scope: std::marker::PhantomData,
    })
  }
}

impl<Args: JsValuesTupleIntoVec, Return> Drop for FunctionRef<Args, Return> {
  fn drop(&mut self) {
    let status = unsafe { sys::napi_delete_reference(self.env, self.inner) };
    debug_assert_eq!(status, sys::Status::napi_ok, "Drop FunctionRef failed");
  }
}

impl<Args: JsValuesTupleIntoVec, Return> TypeName for FunctionRef<Args, Return> {
  fn type_name() -> &'static str {
    "Function"
  }

  fn value_type() -> crate::ValueType {
    ValueType::Function
  }
}

impl<Args: JsValuesTupleIntoVec, Return> FromNapiValue for FunctionRef<Args, Return> {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> Result<Self> {
    let mut reference = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_create_reference(env, value, 1, &mut reference) },
      "Create reference failed"
    )?;
    Ok(FunctionRef {
      inner: reference,
      env,
      _args: std::marker::PhantomData,
      _return: std::marker::PhantomData,
    })
  }
}

impl<Args: JsValuesTupleIntoVec, Return: FromNapiValue> ValidateNapiValue
  for FunctionRef<Args, Return>
{
}

pub struct FunctionCallContext<'scope> {
  pub(crate) args: &'scope [sys::napi_value],
  pub(crate) this: sys::napi_value,
  pub env: &'scope mut Env,
}

impl FunctionCallContext<'_> {
  /// Get the number of arguments from the JavaScript function call.
  pub fn length(&self) -> usize {
    self.args.len()
  }

  pub fn get<ArgType: FromNapiValue>(&self, index: usize) -> Result<ArgType> {
    if index >= self.length() {
      Err(crate::Error::new(
        crate::Status::GenericFailure,
        "Arguments index out of range".to_owned(),
      ))
    } else {
      unsafe { ArgType::from_napi_value(self.env.0, self.args[index]) }
    }
  }

  pub fn try_get<ArgType: NapiValue + TypeName + FromNapiValue>(
    &self,
    index: usize,
  ) -> Result<Either<ArgType, JsUndefined>> {
    let len = self.length();
    if index >= len {
      Err(crate::Error::new(
        crate::Status::GenericFailure,
        "Arguments index out of range".to_owned(),
      ))
    } else if index < len {
      unsafe { ArgType::from_raw(self.env.0, self.args[index]) }.map(Either::A)
    } else {
      self.env.get_undefined().map(Either::B)
    }
  }

  /// Get the first argument from the JavaScript function call.
  pub fn first_arg<T: FromNapiValue>(&self) -> Result<T> {
    if self.args.is_empty() {
      return Err(crate::Error::new(
        crate::Status::InvalidArg,
        "There is no arguments",
      ));
    }
    unsafe { T::from_napi_value(self.env.0, self.args[0]) }
  }

  /// Get the arguments from the JavaScript function call.
  /// The arguments will be converted to a tuple.
  /// If the number of arguments is not equal to the number of tuple elements, an error will be returned.
  /// example:
  /// ```rust
  /// let (num, string) = ctx.args::<(u32, String)>()?;
  /// ````
  pub fn args<Args: TupleFromSliceValues>(&self) -> Result<Args> {
    unsafe { Args::from_slice_values(self.env.0, self.args) }
  }

  /// Get the arguments Vec from the JavaScript function call.
  pub fn arguments<T: FromNapiValue>(&self) -> Result<Vec<T>> {
    self
      .args
      .iter()
      .map(|arg| unsafe { <T as FromNapiValue>::from_napi_value(self.env.0, *arg) })
      .collect::<Result<Vec<T>>>()
  }

  /// Get the `this` from the JavaScript function call.
  pub fn this<This: FromNapiValue>(&self) -> Result<This> {
    unsafe { This::from_napi_value(self.env.0, self.this) }
  }
}
