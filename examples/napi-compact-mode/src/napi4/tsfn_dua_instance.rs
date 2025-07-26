use std::sync::Arc;

use napi_derive_ohos::js_function;
use napi_ohos::{
  bindgen_prelude::Function, threadsafe_function::ThreadsafeFunction, CallContext, JsObject, Status,
};

#[derive(Clone)]
pub struct A {
  pub cb: Arc<ThreadsafeFunction<String, napi_ohos::Unknown<'static>, String, Status, false, true>>,
}

#[js_function(1)]
pub fn constructor(ctx: CallContext) -> napi_ohos::Result<()> {
  let callback = ctx.get::<Function<String>>(0)?;

  let cb: Arc<ThreadsafeFunction<String, napi_ohos::Unknown, String, Status, false, true>> =
    Arc::new(
      callback
        .build_threadsafe_function()
        .weak::<true>()
        .build()?,
    );

  let mut this: JsObject = ctx.this_unchecked();
  let obj = A { cb };

  ctx.env.wrap(&mut this, obj, None)?;
  Ok(())
}

#[js_function]
pub fn call(ctx: CallContext) -> napi_ohos::Result<()> {
  let this = ctx.this_unchecked();
  let obj = ctx.env.unwrap::<A>(&this)?;
  obj.cb.call(
    "ThreadsafeFunction NonBlocking Call".to_owned(),
    napi_ohos::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
  );
  Ok(())
}
