use std::env;

fn main() {
  if env::var("CARGO_FEATURE_NAPI9").is_ok() {
    panic!("Please don't set features with napi9")
  }

  if env::var("CARGO_FEATURE_EXPERIMENTAL").is_ok() {
    panic!("Please don't set features with experimental")
  }
}
