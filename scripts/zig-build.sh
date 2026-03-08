#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." &>/dev/null && pwd)"
TARGET="${1:-aarch64-unknown-linux-ohos}"

cd "${REPO_ROOT}/examples/hello"
cargo zigbuild --target "${TARGET}"

cd "${REPO_ROOT}/examples/napi"
cargo zigbuild --target "${TARGET}"

cd "${REPO_ROOT}/examples/napi-compact-mode"
cargo zigbuild --target "${TARGET}"
