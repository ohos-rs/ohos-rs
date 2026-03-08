#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." &>/dev/null && pwd)"

LEGACY_ARKVM_DIR="${ARKVM_X64_STATIC_DIR:-}"
ARK_HOST_TOOLS_DIR="${ARK_HOST_TOOLS_DIR:-${ARK_HYBRID_HOST_DIR:-${LEGACY_ARKVM_DIR:-}}}"
REQUIRE_ARK_HOST_TOOLS="${REQUIRE_ARK_HOST_TOOLS:-0}"

if [[ -z "${ARK_HOST_TOOLS_DIR}" ]]; then
  if [[ "${REQUIRE_ARK_HOST_TOOLS}" == "1" ]]; then
    echo "==> Missing ARK_HOST_TOOLS_DIR: CI must provide an ArkJS N-API host bundle" >&2
    exit 1
  fi
  echo "==> Skip ArkTS + N-API tests (set ARK_HOST_TOOLS_DIR to the ArkJS N-API host bundle to enable)"
  exit 0
fi

if [[ ! -d "${ARK_HOST_TOOLS_DIR}" ]]; then
  if [[ "${REQUIRE_ARK_HOST_TOOLS}" == "1" ]]; then
    echo "==> ARK_HOST_TOOLS_DIR not found: ${ARK_HOST_TOOLS_DIR}" >&2
    exit 1
  fi
  echo "==> Skip ArkTS + N-API tests (ARK_HOST_TOOLS_DIR not found: ${ARK_HOST_TOOLS_DIR})"
  exit 0
fi

echo "==> Run ArkTS + N-API tests"
"${REPO_ROOT}/scripts/ohos/run_split_tests_arkvm.sh"
