#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." &>/dev/null && pwd)"

export CARGO_REGISTRIES_CRATES_IO_PROTOCOL="${CARGO_REGISTRIES_CRATES_IO_PROTOCOL:-sparse}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target/arkvm-host}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to build ArkVM host libs" >&2
  exit 127
fi

cargo build --locked -p example --release --features "${EXAMPLE_BUILD_FEATURES:-arkvm-test}"
cargo build --locked -p compact --release --features "${COMPACT_BUILD_FEATURES:-arkvm-test}"
cargo build --locked --manifest-path third_party/openharmony/buffer/Cargo.toml --release --features arkvm-test
cargo build --locked --manifest-path third_party/openharmony/worker/Cargo.toml --release --features arkvm-test
