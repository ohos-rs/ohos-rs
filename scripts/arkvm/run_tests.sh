#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." &>/dev/null && pwd)"

ARK_HOST_BUNDLE_DIR="${ARK_HOST_BUNDLE_DIR:-${ARK_HOST_TOOLS_DIR:-}}"
ARK_ES2ABC="${ARK_ES2ABC:-${ARK_HOST_BUNDLE_DIR}/es2abc}"
ARK_JS_NAPI_CLI="${ARK_JS_NAPI_CLI:-${ARK_HOST_BUNDLE_DIR}/ark_js_napi_cli}"
ARK_ACE_NAPI_LIB="${ARK_ACE_NAPI_LIB:-${ARK_HOST_BUNDLE_DIR}/libace_napi.so}"
ARK_STUB_FILE="${ARK_STUB_FILE:-${ARK_HOST_BUNDLE_DIR}/stub.an}"
TEST_TIMEOUT_SEC="${TEST_TIMEOUT_SEC:-90}"
KEEP_WORKDIR="${KEEP_WORKDIR:-0}"
FAIL_ON_CASE_ERROR="${FAIL_ON_CASE_ERROR:-1}"

WORK_ROOT="${ARK_HOST_WORK_ROOT:-${REPO_ROOT}/.tmp_arkvm_runner}"
WORKSPACE_DIR="${WORK_ROOT}/workspace"
TEST_ROOT="${WORKSPACE_DIR}/test"
RESULT_FILE="${WORK_ROOT}/results.tsv"
TARGET_RELEASE_DIR="${TARGET_RELEASE_DIR:-${REPO_ROOT}/target/arkvm-host/release}"

require_host_bundle() {
  [[ -n "${ARK_HOST_BUNDLE_DIR}" ]] || {
    echo "ARK_HOST_TOOLS_DIR or ARK_HOST_BUNDLE_DIR is required" >&2
    exit 1
  }
  [[ -d "${ARK_HOST_BUNDLE_DIR}" ]] || { echo "Ark host bundle not found: ${ARK_HOST_BUNDLE_DIR}" >&2; exit 1; }
  [[ -x "${ARK_ES2ABC}" ]] || { echo "Missing binary: ${ARK_ES2ABC}" >&2; exit 1; }
  [[ -x "${ARK_JS_NAPI_CLI}" ]] || { echo "Missing binary: ${ARK_JS_NAPI_CLI}" >&2; exit 1; }
  [[ -f "${ARK_ACE_NAPI_LIB}" ]] || { echo "Missing shared lib: ${ARK_ACE_NAPI_LIB}" >&2; exit 1; }

  if [[ ! -f "${ARK_STUB_FILE}" ]]; then
    echo "==> Stub file not found, continue without --stub-file: ${ARK_STUB_FILE}"
  fi
}

require_host_libs() {
  local lib
  for lib in libexample.so libcompact.so libbuffer.so libworker.so; do
    [[ -f "${TARGET_RELEASE_DIR}/${lib}" ]] || {
      echo "Missing host test library: ${TARGET_RELEASE_DIR}/${lib}" >&2
      echo "Build host test libraries before running this script." >&2
      exit 1
    }
  done
}

prepare_workspace() {
  rm -rf "${WORK_ROOT}"
  mkdir -p "${WORKSPACE_DIR}" "${WORK_ROOT}/logs"
  cp -R "${REPO_ROOT}/test" "${WORKSPACE_DIR}/"
  [[ -d "${REPO_ROOT}/third_party" ]] && cp -R "${REPO_ROOT}/third_party" "${WORKSPACE_DIR}/"
  cat > "${TEST_ROOT}/runtime/ark_host_config.ts" <<EOF
export const ARK_HOST_BUNDLE_DIR = "${ARK_HOST_BUNDLE_DIR}";
EOF
  rm -rf "${TEST_ROOT}/suites" "${TEST_ROOT}/suites.list"
}

run_compile() {
  ARK_HOST_BUNDLE_DIR="${ARK_HOST_BUNDLE_DIR}" \
    WORK_ROOT="${WORK_ROOT}" \
    WORKSPACE_DIR="${WORKSPACE_DIR}" \
    TEST_ROOT="${TEST_ROOT}" \
    bash "${SCRIPT_DIR}/compile_abc.sh"
}

run_suites() {
  echo "==> Ark host bundle: ${ARK_HOST_BUNDLE_DIR}"
  echo "==> Work root: ${WORK_ROOT}"
  ARK_HOST_BUNDLE_DIR="${ARK_HOST_BUNDLE_DIR}" \
    WORK_ROOT="${WORK_ROOT}" \
    WORKSPACE_DIR="${WORKSPACE_DIR}" \
    TEST_ROOT="${TEST_ROOT}" \
    RESULT_FILE="${RESULT_FILE}" \
    STUB_FILE="${ARK_STUB_FILE}" \
    TARGET_RELEASE_DIR="${TARGET_RELEASE_DIR}" \
    TEST_TIMEOUT_SEC="${TEST_TIMEOUT_SEC}" \
    bash "${SCRIPT_DIR}/run_suites.sh"
}

finalize() {
  local failed_count=0

  echo "Results: ${RESULT_FILE}"
  echo
  cat "${RESULT_FILE}"

  if [[ "${FAIL_ON_CASE_ERROR}" == "1" ]]; then
    failed_count="$(grep -E '^summary_failed=' "${RESULT_FILE}" | cut -d'=' -f2 | tail -n1)"
  fi

  if [[ "${KEEP_WORKDIR}" != "1" ]]; then
    rm -rf "${WORK_ROOT}"
  fi

  if [[ "${FAIL_ON_CASE_ERROR}" == "1" ]]; then
    [[ "${failed_count:-0}" -eq 0 ]]
  fi
}

require_host_bundle
require_host_libs
prepare_workspace
bash "${SCRIPT_DIR}/split_tests.sh" "${TEST_ROOT}/source" "${TEST_ROOT}"
run_compile
run_suites
finalize
