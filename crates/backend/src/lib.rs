#[macro_use]
extern crate quote;

use std::collections::HashMap;
use std::sync::LazyLock;

use proc_macro2::TokenStream;

#[macro_use]
pub mod error;
pub mod ast;
pub mod codegen;
#[cfg(feature = "type-def")]
pub mod typegen;

pub use ast::*;
pub use codegen::*;
pub use error::{BindgenResult, Diagnostic};
#[cfg(feature = "type-def")]
pub use typegen::*;

#[derive(Debug)]
pub struct Napi {
  pub item: NapiItem,
}

macro_rules! napi_ast_impl {
  ( $( ($v:ident, $ast:ident), )* ) => {
    #[derive(Debug)]
    #[allow(clippy::large_enum_variant)]
    pub enum NapiItem {
      $($v($ast)),*
    }

    impl TryToTokens for Napi {
      fn try_to_tokens(&self, tokens: &mut TokenStream) -> BindgenResult<()> {
        match self.item {
          $( NapiItem::$v(ref ast) => ast.try_to_tokens(tokens) ),*
        }
      }
    }

		#[cfg(feature = "type-def")]
		impl ToTypeDef for Napi {
			fn to_type_def(&self) -> Option<TypeDef> {
				match self.item {
          $( NapiItem::$v(ref ast) => ast.to_type_def() ),*
        }
			}
		}

    impl Napi {
      pub fn register_name(&self) -> String {
        match self.item {
          $( NapiItem::$v(ref ast) => ast.register_name.to_string() ),*
        }
      }
    }
  };
}

napi_ast_impl! {
 (Fn, NapiFn),
 (Struct, NapiStruct),
 (Impl, NapiImpl),
 (Enum, NapiEnum),
 (Const, NapiConst),
 (Type, NapiType),
}

pub(crate) static PRIMITIVE_TYPES: &[(&str, (&str, bool, bool))] = &[
  ("JsUndefined", ("undefined", false, false)),
  ("()", ("undefined", false, false)),
  ("Undefined", ("undefined", false, false)),
  ("JsNumber", ("number", false, false)),
  ("i8", ("number", false, false)),
  ("i16", ("number", false, false)),
  ("i32", ("number", false, false)),
  ("i64", ("number", false, false)),
  ("f32", ("number", false, false)),
  ("f64", ("number", false, false)),
  ("u8", ("number", false, false)),
  ("u16", ("number", false, false)),
  ("u32", ("number", false, false)),
  // serde `Number`
  ("Number", ("number", false, false)),
  ("u64", ("bigint", false, false)),
  ("i64n", ("bigint", false, false)),
  ("u128", ("bigint", false, false)),
  ("i128", ("bigint", false, false)),
  ("usize", ("bigint", false, false)),
  ("isize", ("bigint", false, false)),
  ("JsBigInt", ("bigint", false, false)),
  ("BigInt", ("bigint", false, false)),
  ("JsBoolean", ("boolean", false, false)),
  ("bool", ("boolean", false, false)),
  ("JsString", ("string", false, false)),
  ("String", ("string", false, false)),
  ("RawCString", ("string", false, false)),
  ("str", ("string", false, false)),
  ("Latin1String", ("string", false, false)),
  ("Utf16String", ("string", false, false)),
  ("char", ("string", false, false)),
  ("Null", ("null", false, false)),
  ("JsNull", ("null", false, false)),
  ("null", ("null", false, false)),
  ("Symbol", ("symbol", false, false)),
  ("JsSymbol", ("symbol", false, false)),
  ("SymbolRef", ("symbol", false, false)),
  ("JsFunction", ("(...args: any[]) => any", true, false)),
];

pub(crate) static TYPEDARRAY_SLICE_TYPES: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
  HashMap::from([
    ("u8", "Uint8Array"),
    ("i8", "Int8Array"),
    ("u16", "Uint16Array"),
    ("i16", "Int16Array"),
    ("u32", "Uint32Array"),
    ("i32", "Int32Array"),
    ("f32", "Float32Array"),
    ("f64", "Float64Array"),
    ("u64", "BigUint64Array"),
    ("i64", "BigInt64Array"),
  ])
});
