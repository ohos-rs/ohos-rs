#![allow(unused_variables)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::zero_repeat_side_effects)]
#![allow(deprecated)]

#[macro_use]
extern crate napi_derive_ohos;
#[macro_use]
extern crate serde_derive;

use napi_ohos::{Env, JsObject, Result};

mod cleanup_env;
mod napi4;
mod napi5;
mod napi6;
mod napi7;
mod napi8;
mod tokio_rt;

mod array;
mod arraybuffer;
mod buffer;
mod class;
mod either;
mod env;
mod error;
mod external;
mod function;
mod global;
mod napi_version;
mod object;
mod serde;
mod string;

use napi_version::get_napi_version;

#[module_exports]
fn init(mut exports: JsObject, env: Env) -> Result<()> {
  exports.create_named_method("getNapiVersion", get_napi_version)?;
  array::register_js(&mut exports)?;
  error::register_js(&mut exports)?;
  string::register_js(&mut exports)?;
  serde::register_js(&mut exports)?;
  external::register_js(&mut exports)?;
  arraybuffer::register_js(&mut exports)?;
  buffer::register_js(&mut exports)?;
  either::register_js(&mut exports)?;
  function::register_js(&mut exports)?;
  class::register_js(&mut exports)?;
  env::register_js(&mut exports)?;
  object::register_js(&mut exports)?;
  global::register_js(&mut exports)?;
  cleanup_env::register_js(&mut exports)?;
  napi4::register_js(&mut exports, &env)?;
  tokio_rt::register_js(&mut exports)?;
  napi5::register_js(&mut exports)?;
  napi6::register_js(&mut exports)?;
  napi7::register_js(&mut exports)?;
  napi8::register_js(&mut exports)?;
  Ok(())
}
