use napi_ohos::{
  bindgen_prelude::{Either, Function, Promise},
  threadsafe_function::ThreadsafeFunction,
  Result, Status,
};
use std::sync::Arc;

#[napi]
pub type CustomU32 = u32;

#[napi]
pub type MyPromise = Either<String, Promise<String>>;

#[napi]
pub type Nullable<T> = Option<T>;

#[napi(js_name = "VoidNullable<T = void>")]
pub type VoidNullable<T> = Nullable<T>;

#[napi]
pub type RuleHandler<'a, Args, Ret> = Function<'a, Args, Ret>;

#[napi(object, object_to_js = false)]
pub struct Rule<'a> {
  pub name: String,
  pub handler: RuleHandler<'a, u32, u32>,
}

#[napi]
pub fn call_rule_handler(rule: Rule, arg: u32) -> Result<u32> {
  rule.handler.call(arg)
}

#[napi(object)]
pub struct PluginLoadResult {
  pub name: String,
  pub version: String,
}

// Test fixture for ThreadsafeFunction with single argument (issue #2726)
#[napi]
pub type ExternalLinterLoadPluginCb =
  Arc<ThreadsafeFunction<String, PluginLoadResult, String, Status, false>>;

#[napi]
#[allow(unused_parens)]
pub type ExternalLinterLoadPluginCb2 =
  Arc<ThreadsafeFunction<(String), PluginLoadResult, (String), Status, false>>;
