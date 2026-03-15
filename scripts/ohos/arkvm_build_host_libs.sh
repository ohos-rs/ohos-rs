#!/usr/bin/env bash
set -euo pipefail

export CARGO_REGISTRIES_CRATES_IO_PROTOCOL="${CARGO_REGISTRIES_CRATES_IO_PROTOCOL:-sparse}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/work/target/arkvm-host}"

cargo build --locked -p example --release --features "${EXAMPLE_BUILD_FEATURES:-arkvm-test}"
cargo build --locked -p compact --release --features "${COMPACT_BUILD_FEATURES:-arkvm-test}"
cargo build --locked --manifest-path third_party/openharmony/buffer/Cargo.toml --release --features arkvm-test
cargo build --locked --manifest-path third_party/openharmony/worker/Cargo.toml --release --features arkvm-test
