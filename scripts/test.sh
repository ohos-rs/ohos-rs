#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." &>/dev/null && pwd)"

cd "${REPO_ROOT}/examples/hello"
ohrs build

cd "${REPO_ROOT}/examples/napi"
ohrs build

cd "${REPO_ROOT}/examples/napi-compact-mode"
ohrs build
