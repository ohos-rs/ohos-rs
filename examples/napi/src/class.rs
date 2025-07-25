use napi_ohos::{
  bindgen_prelude::{
    Buffer, ClassInstance, Function, JavaScriptClassExt, JsObjectValue, JsValue, ObjectFinalize,
    This, Uint8Array, Unknown,
  },
  Env, Property, PropertyAttributes, Result,
};

use crate::r#enum::Kind;

/// `constructor` option for `struct` requires all fields to be public,
/// otherwise tag impl fn as constructor
/// #[napi(constructor)]
#[napi]
pub struct Animal {
  #[napi(readonly)]
  /// Kind of animal
  pub kind: Kind,

  name: String,
}

#[napi]
impl Animal {
  /// This is the constructor
  #[napi(constructor)]
  pub fn new(kind: Kind, name: String) -> Self {
    Animal { kind, name }
  }

  /// This is a factory method
  #[napi(factory)]
  pub fn with_kind(kind: Kind) -> Self {
    Animal {
      kind,
      name: "Default".to_owned(),
    }
  }

  #[napi(getter)]
  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }

  #[napi(setter)]
  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }

  #[napi(getter, js_name = "type")]
  pub fn kind(&self) -> Kind {
    self.kind
  }

  #[napi(setter, js_name = "type")]
  pub fn set_kind(&mut self, kind: Kind) {
    self.kind = kind;
  }

  /// This is a
  /// multi-line comment
  /// with an emoji 🚀
  #[napi]
  pub fn whoami(&self) -> String {
    match self.kind {
      Kind::Dog => {
        format!("Dog: {}", self.name)
      }
      Kind::Cat => format!("Cat: {}", self.name),
      Kind::Duck => format!("Duck: {}", self.name),
    }
  }

  #[napi]
  /// This is static...
  pub fn get_dog_kind() -> Kind {
    Kind::Dog
  }

  #[napi]
  /// Here are some characters and character sequences
  /// that should be escaped correctly:
  /// \[]{}/\:""{
  /// }
  pub fn return_other_class(&self) -> Dog {
    Dog {
      name: "Doge".to_owned(),
    }
  }

  #[napi]
  pub fn return_other_class_with_custom_constructor(&self) -> Bird {
    Bird::new("parrot".to_owned())
  }

  #[napi]
  pub fn override_individual_arg_on_method(
    &self,
    normal_ty: String,
    #[napi(ts_arg_type = "{n: string}")] overridden_ty: napi_ohos::bindgen_prelude::Object,
  ) -> Bird {
    let obj = overridden_ty.coerce_to_object().unwrap();
    let the_n: Option<String> = obj.get("n").unwrap();

    Bird::new(format!("{}-{}", normal_ty, the_n.unwrap()))
  }
}

#[napi(constructor)]
pub struct Dog {
  pub name: String,
}

#[cfg_attr(not(feature = "cfg_attr_napi"), napi_derive_ohos::napi)]
pub struct Bird {
  pub name: String,
}

#[cfg_attr(not(feature = "cfg_attr_napi"), napi_derive_ohos::napi)]
impl Bird {
  #[cfg_attr(not(feature = "cfg_attr_napi"), napi_derive_ohos::napi(constructor))]
  pub fn new(name: String) -> Self {
    Bird { name }
  }

  #[cfg_attr(not(feature = "cfg_attr_napi"), napi_derive_ohos::napi)]
  pub fn get_count(&self) -> u32 {
    1234
  }

  #[cfg_attr(not(feature = "cfg_attr_napi"), napi_derive_ohos::napi)]
  pub async fn get_name_async(&self) -> &str {
    tokio::time::sleep(std::time::Duration::new(1, 0)).await;
    self.name.as_str()
  }

  #[cfg_attr(not(feature = "cfg_attr_napi"), napi_derive_ohos::napi)]
  pub fn accept_slice_method(&self, slice: &[u8]) -> u32 {
    slice.len() as u32
  }
}

/// Smoking test for type generation
#[napi]
#[repr(transparent)]
pub struct Blake2bHasher(u32);

#[napi]
impl Blake2bHasher {
  #[napi(factory)]
  pub fn with_key(key: &Blake2bKey) -> Self {
    Blake2bHasher(key.get_inner())
  }
}

#[napi]
impl Blake2bHasher {
  #[napi]
  pub fn update(&mut self, data: Buffer) {
    self.0 += data.len() as u32;
  }
}

#[napi]
pub struct Blake2bKey(u32);

impl Blake2bKey {
  fn get_inner(&self) -> u32 {
    self.0
  }
}

#[napi]
pub struct Context {
  data: String,
  pub maybe_need: Option<bool>,
  pub buffer: Uint8Array,
}

// Test for return `napi::Result` and `Result`
#[napi]
impl Context {
  #[napi(constructor)]
  pub fn new() -> napi_ohos::Result<Self> {
    Ok(Self {
      data: "not empty".into(),
      maybe_need: None,
      buffer: Uint8Array::new(vec![0, 1, 2, 3]),
    })
  }

  #[napi(factory)]
  pub fn with_data(data: String) -> Result<Self> {
    Ok(Self {
      data,
      maybe_need: Some(true),
      buffer: Uint8Array::new(vec![0, 1, 2, 3]),
    })
  }

  #[napi(factory)]
  pub fn with_buffer(buf: Uint8Array) -> Self {
    Self {
      data: "not empty".into(),
      maybe_need: None,
      buffer: buf,
    }
  }

  #[napi]
  pub fn method(&self) -> String {
    self.data.clone()
  }
}

#[napi(constructor)]
pub struct AnimalWithDefaultConstructor {
  pub name: String,
  pub kind: u32,
}

// Test for skip_typescript
#[napi]
pub struct NinjaTurtle {
  pub name: String,
  #[napi(skip_typescript)]
  pub mask_color: String,
}

#[napi]
impl NinjaTurtle {
  #[napi]
  pub fn is_instance_of(env: Env, value: Unknown) -> Result<bool> {
    Self::instance_of(&env, &value)
  }

  /// Create your ninja turtle! 🐢
  #[napi(factory)]
  pub fn new_raph() -> Self {
    Self {
      name: "Raphael".to_owned(),
      mask_color: "Red".to_owned(),
    }
  }

  /// We are not going to expose this character, so we just skip it...
  #[napi(factory, skip_typescript)]
  pub fn new_leo() -> Self {
    Self {
      name: "Leonardo".to_owned(),
      mask_color: "Blue".to_owned(),
    }
  }

  #[napi]
  pub fn get_mask_color(&self) -> &str {
    self.mask_color.as_str()
  }

  #[napi]
  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }

  #[napi]
  pub fn return_this<'scope>(&'scope self, this: This<'scope>) -> This<'scope> {
    this
  }
}

#[napi(js_name = "Assets")]
pub struct JsAssets {}

#[napi]
impl JsAssets {
  #[napi(constructor)]
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    JsAssets {}
  }

  #[napi]
  pub fn get(&mut self, _id: u32) -> Option<JsAsset> {
    Some(JsAsset {})
  }
}

#[napi(js_name = "Asset")]
pub struct JsAsset {}

#[napi]
impl JsAsset {
  #[napi(constructor)]
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self {}
  }

  #[napi(getter)]
  pub fn get_file_path(&self) -> u32 {
    1
  }
}

#[napi]
pub struct Optional {}

#[napi]
impl Optional {
  #[napi]
  pub fn option_end(required: String, optional: Option<String>) -> String {
    match optional {
      None => required,
      Some(optional) => format!("{} {}", required, optional),
    }
  }

  #[napi]
  pub fn option_start(optional: Option<String>, required: String) -> String {
    match optional {
      None => required,
      Some(optional) => format!("{} {}", optional, required),
    }
  }

  #[napi]
  pub fn option_start_end(
    optional1: Option<String>,
    required: String,
    optional2: Option<String>,
  ) -> String {
    match (optional1, optional2) {
      (None, None) => required,
      (None, Some(optional2)) => format!("{} {}", required, optional2),
      (Some(optional1), None) => format!("{} {}", optional1, required),
      (Some(optional1), Some(optional2)) => format!("{} {} {}", optional1, required, optional2),
    }
  }

  #[napi]
  pub fn option_only(optional: Option<String>) -> String {
    match optional {
      None => "".to_string(),
      Some(optional) => optional,
    }
  }
}

#[napi(object)]
pub struct ObjectFieldClassInstance<'env> {
  pub bird: ClassInstance<'env, Bird>,
}

#[napi]
pub fn create_object_with_class_field(env: &Env) -> Result<ObjectFieldClassInstance<'_>> {
  Ok(ObjectFieldClassInstance {
    bird: Bird {
      name: "Carolyn".to_owned(),
    }
    .into_instance(env)?,
  })
}

#[napi]
pub fn receive_object_with_class_field(
  object: ObjectFieldClassInstance,
) -> Result<ClassInstance<Bird>> {
  Ok(object.bird)
}

#[napi(constructor)]
pub struct NotWritableClass {
  #[napi(writable = false)]
  pub name: String,
}

#[napi]
impl NotWritableClass {
  #[napi(writable = false)]
  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }
}

#[napi(custom_finalize)]
pub struct CustomFinalize {
  width: u32,
  height: u32,
  inner: Vec<u8>,
}

#[napi]
impl CustomFinalize {
  #[napi(constructor)]
  pub fn new(env: Env, width: u32, height: u32) -> Result<Self> {
    let inner = vec![0; (width * height * 4) as usize];
    let inner_size = inner.len();
    env.adjust_external_memory(inner_size as i64)?;
    Ok(Self {
      width,
      height,
      inner,
    })
  }
}

impl ObjectFinalize for CustomFinalize {
  fn finalize(self, env: Env) -> Result<()> {
    env.adjust_external_memory(-(self.inner.len() as i64))?;
    Ok(())
  }
}

#[napi(constructor)]
pub struct Width {
  pub value: i32,
}

#[napi]
pub fn plus_one(this: This<&Width>) -> i32 {
  this.object.value + 1
}

#[napi]
pub struct GetterSetterWithClosures {}

#[napi]
impl GetterSetterWithClosures {
  #[napi(constructor)]
  pub fn new(_env: &Env, mut this: This) -> Result<Self> {
    this.define_properties(&[
      Property::new()
        .with_utf8_name("name")?
        .with_setter_closure(move |_env, mut this, value: String| {
          this.set_named_property("_name", format!("I'm {}", value))?;
          Ok(())
        })
        .with_getter_closure(|_env, this| this.get_named_property_unchecked::<Unknown>("_name")),
      Property::new()
        .with_utf8_name("age")?
        .with_getter_closure(|_env, _this| Ok(0.3)),
    ])?;

    Ok(Self {})
  }
}

#[napi]
pub struct CatchOnConstructor {}

#[napi]
impl CatchOnConstructor {
  #[napi(constructor, catch_unwind)]
  pub fn new() -> Self {
    Self {}
  }
}

#[napi]
pub struct CatchOnConstructor2 {}

#[napi]
impl CatchOnConstructor2 {
  #[napi(constructor, catch_unwind)]
  pub fn new() -> Self {
    panic!("CatchOnConstructor2 panic");
  }
}

#[napi]
pub struct ClassWithLifetime<'a> {
  inner: ClassInstance<'a, Animal>,
  inner2: ClassInstance<'a, Animal>,
}

#[napi]
impl<'scope> ClassWithLifetime<'scope> {
  #[napi(constructor)]
  pub fn new(env: &Env, mut this: This<'scope>) -> Result<Self> {
    let instance = Animal {
      kind: Kind::Cat,
      name: "alie".to_owned(),
    }
    .into_instance(env)?;
    let inner = instance.assign_to_this("inner", &mut this)?;
    let inner2 =
      instance.assign_to_this_with_attributes("inner2", PropertyAttributes::Default, &mut this)?;
    Ok(Self { inner, inner2 })
  }

  #[napi]
  pub fn get_name(&self) -> &str {
    self.inner.get_name()
  }
}

#[napi(js_name = "MyJsNamedClass")]
pub struct OriginalRustNameForJsNamedStruct {
  value: String,
}

#[napi]
impl OriginalRustNameForJsNamedStruct {
  #[napi(constructor)]
  pub fn new(value: String) -> Self {
    OriginalRustNameForJsNamedStruct { value }
  }

  #[napi]
  pub fn get_value(&self) -> String {
    self.value.clone()
  }

  #[napi]
  pub fn multiply_value(&self, times: u32) -> String {
    self.value.repeat(times as usize)
  }
}

// Test case for js_name struct with methods only (no constructor)
#[napi(js_name = "JSOnlyMethodsClass")]
pub struct RustOnlyMethodsClass {
  pub data: String,
}

#[napi]
impl RustOnlyMethodsClass {
  #[napi]
  pub fn process_data(&self) -> String {
    format!("processed: {}", self.data)
  }

  #[napi]
  pub fn get_length(&self) -> u32 {
    self.data.len() as u32
  }
}

// Test case for issue #2746: instanceof failure for objects returned from getters
#[napi]
pub struct Thing;

#[napi]
pub struct ThingList;

#[napi]
impl ThingList {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self
  }

  #[napi(getter)]
  pub fn thing() -> Thing {
    Thing
  }
}

#[napi(
  ts_return_type = r#"typeof DynamicRustClass\n\nclass DynamicRustClass {
  constructor(value: number)
  rustMethod(): number
}"#
)]
pub fn define_class(env: &Env) -> Result<Function> {
  env.define_class(
    "DynamicRustClass",
    rust_class_constructor_c_callback,
    &[Property::new()
      .with_utf8_name("rustMethod")?
      .with_method(rust_class_method_c_callback)],
  )
}

#[napi(no_export)]
fn rust_class_constructor(value: i32, mut this: This) -> Result<()> {
  this.set_named_property("dynamicValue", value)?;
  Ok(())
}

#[napi(no_export)]
fn rust_class_method(this: This) -> Result<i32> {
  this.get_named_property_unchecked::<i32>("dynamicValue")
}
