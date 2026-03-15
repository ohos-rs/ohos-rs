#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." &>/dev/null && pwd)"

ARK_HOST_TOOLS_DIR="${ARK_HOST_TOOLS_DIR:-/Users/ranger/Desktop/x64_linux_static}"
ARK_HOST_BUNDLE_DIR="${ARK_HOST_BUNDLE_DIR:-${ARK_HOST_TOOLS_DIR}}"
ARK_ES2ABC="${ARK_ES2ABC:-${ARK_HOST_BUNDLE_DIR}/es2abc}"
ARK_JS_NAPI_CLI="${ARK_JS_NAPI_CLI:-${ARK_HOST_BUNDLE_DIR}/ark_js_napi_cli}"
ARK_ACE_NAPI_LIB="${ARK_ACE_NAPI_LIB:-${ARK_HOST_BUNDLE_DIR}/libace_napi.so}"
ARK_STUB_FILE="${ARK_STUB_FILE:-${ARK_HOST_BUNDLE_DIR}/stub.an}"
ARK_HOST_CONTAINER_MOUNT="${ARK_HOST_CONTAINER_MOUNT:-/ark-host}"

ARKVM_DOCKER_IMAGE="${ARKVM_DOCKER_IMAGE:-ohos-rs-arkvm-linux-x64:latest}"
DOCKER_PLATFORM="${DOCKER_PLATFORM:-linux/amd64}"
TEST_TIMEOUT_SEC="${TEST_TIMEOUT_SEC:-90}"
KEEP_WORKDIR="${KEEP_WORKDIR:-0}"
FAIL_ON_CASE_ERROR="${FAIL_ON_CASE_ERROR:-1}"
EXAMPLE_BUILD_FEATURES="${EXAMPLE_BUILD_FEATURES:-arkvm-test}"
COMPACT_BUILD_FEATURES="${COMPACT_BUILD_FEATURES:-arkvm-test}"

ARK_HOST_CACHE_ROOT="${ARK_HOST_CACHE_ROOT:-${REPO_ROOT}/.cache/ark-host}"
TMPDIR="${ARK_HOST_TMPDIR:-${ARK_HOST_CACHE_ROOT}/tmp}"
WORK_ROOT="${ARK_HOST_WORK_ROOT:-${REPO_ROOT}/.tmp_ohos_split_runner}"
WORKSPACE_DIR="${WORK_ROOT}/workspace"
TEST_ROOT="${WORKSPACE_DIR}/test/ohos"
RESULT_FILE="${WORK_ROOT}/results.tsv"
TARGET_RELEASE_DIR_HOST="${REPO_ROOT}/target/arkvm-host/release"
CONTAINER_WORK_ROOT="/work/.tmp_ohos_split_runner"
CONTAINER_WORKSPACE_DIR="${CONTAINER_WORK_ROOT}/workspace"
CONTAINER_TEST_ROOT="${CONTAINER_WORKSPACE_DIR}/test/ohos"
CONTAINER_RESULT_FILE="${CONTAINER_WORK_ROOT}/results.tsv"
TARGET_RELEASE_DIR_CONTAINER="/work/target/arkvm-host/release"

if [[ -n "${ARKVM_USE_DOCKER:-}" ]]; then
  USE_DOCKER="${ARKVM_USE_DOCKER}"
elif [[ "${GITHUB_ACTIONS:-false}" == "true" ]]; then
  USE_DOCKER=0
else
  USE_DOCKER=1
fi

require_host_bundle() {
  mkdir -p "${TMPDIR}" "${ARK_HOST_CACHE_ROOT}"
  export TMPDIR

  [[ -d "${ARK_HOST_BUNDLE_DIR}" ]] || { echo "Ark host bundle not found: ${ARK_HOST_BUNDLE_DIR}" >&2; exit 1; }
  [[ -x "${ARK_ES2ABC}" ]] || { echo "Missing binary: ${ARK_ES2ABC}" >&2; exit 1; }
  [[ -x "${ARK_JS_NAPI_CLI}" ]] || { echo "Missing binary: ${ARK_JS_NAPI_CLI}" >&2; exit 1; }
  [[ -f "${ARK_ACE_NAPI_LIB}" ]] || { echo "Missing shared lib: ${ARK_ACE_NAPI_LIB}" >&2; exit 1; }

  if [[ ! -f "${ARK_STUB_FILE}" ]]; then
    echo "==> Stub file not found, continue without --stub-file: ${ARK_STUB_FILE}"
  fi
}

run_in_docker() {
  docker run --rm \
    --platform "${DOCKER_PLATFORM}" \
    -v "${REPO_ROOT}:/work" \
    -v "${ARK_HOST_BUNDLE_DIR}:${ARK_HOST_CONTAINER_MOUNT}:ro" \
    -w /work \
    -e ARK_HOST_CONTAINER_MOUNT="${ARK_HOST_CONTAINER_MOUNT}" \
    -e WORK_ROOT="${CONTAINER_WORK_ROOT}" \
    -e WORKSPACE_DIR="${CONTAINER_WORKSPACE_DIR}" \
    -e TEST_ROOT="${CONTAINER_TEST_ROOT}" \
    -e RESULT_FILE="${CONTAINER_RESULT_FILE}" \
    -e STUB_FILE="${ARK_HOST_CONTAINER_MOUNT}/stub.an" \
    -e TARGET_RELEASE_DIR="${TARGET_RELEASE_DIR_CONTAINER}" \
    -e TEST_TIMEOUT_SEC="${TEST_TIMEOUT_SEC}" \
    -e EXAMPLE_BUILD_FEATURES="${EXAMPLE_BUILD_FEATURES}" \
    -e COMPACT_BUILD_FEATURES="${COMPACT_BUILD_FEATURES}" \
    -e ARK_HOST_BUNDLE_DIR="${ARK_HOST_CONTAINER_MOUNT}" \
    -e ARK_ACE_NAPI_LIB="${ARK_HOST_CONTAINER_MOUNT}/libace_napi.so" \
    -e CARGO_TARGET_DIR="/work/target/arkvm-host" \
    "${ARKVM_DOCKER_IMAGE}" \
    "$@"
}

run_on_host() {
  env \
    ARK_HOST_CONTAINER_MOUNT="${ARK_HOST_BUNDLE_DIR}" \
    WORK_ROOT="${WORK_ROOT}" \
    WORKSPACE_DIR="${WORKSPACE_DIR}" \
    TEST_ROOT="${TEST_ROOT}" \
    RESULT_FILE="${RESULT_FILE}" \
    STUB_FILE="${ARK_STUB_FILE}" \
    TARGET_RELEASE_DIR="${TARGET_RELEASE_DIR_HOST}" \
    TEST_TIMEOUT_SEC="${TEST_TIMEOUT_SEC}" \
    EXAMPLE_BUILD_FEATURES="${EXAMPLE_BUILD_FEATURES}" \
    COMPACT_BUILD_FEATURES="${COMPACT_BUILD_FEATURES}" \
    ARK_HOST_BUNDLE_DIR="${ARK_HOST_BUNDLE_DIR}" \
    ARK_ACE_NAPI_LIB="${ARK_ACE_NAPI_LIB}" \
    CARGO_TARGET_DIR="${REPO_ROOT}/target/arkvm-host" \
    "$@"
}

run_mode() {
  if [[ "${USE_DOCKER}" == "1" ]]; then
    run_in_docker "$@"
  else
    run_on_host "$@"
  fi
}

build_host_libs() {
  [[ "${SKIP_BUILD_LIBS:-0}" == "1" ]] && return 0
  if [[ "${USE_DOCKER}" == "1" ]]; then
    run_in_docker bash /work/scripts/ohos/arkvm_build_host_libs.sh
  else
    run_on_host bash "${REPO_ROOT}/scripts/ohos/arkvm_build_host_libs.sh"
  fi
}

prepare_workspace() {
  rm -rf "${WORK_ROOT}"
  mkdir -p "${WORKSPACE_DIR}" "${WORK_ROOT}/logs"
  cp -R "${REPO_ROOT}/test" "${WORKSPACE_DIR}/"
  [[ -d "${REPO_ROOT}/third_party" ]] && cp -R "${REPO_ROOT}/third_party" "${WORKSPACE_DIR}/"
  cat > "${TEST_ROOT}/runtime/ark_host_config.ts" <<EOF
export const ARK_HOST_BUNDLE_DIR = "${ARK_HOST_CONTAINER_MOUNT}";
EOF
  rm -rf "${TEST_ROOT}/suites" "${TEST_ROOT}/suites.list"
}

generate_suites() {
  bash "${SCRIPT_DIR}/split_ohos_tests.sh" "${TEST_ROOT}/source" "${TEST_ROOT}"
}

run_compile() {
  if [[ "${USE_DOCKER}" == "1" ]]; then
    run_in_docker bash /work/scripts/ohos/arkvm_compile_abc_in_docker.sh
  else
    run_on_host bash "${REPO_ROOT}/scripts/ohos/arkvm_compile_abc_in_docker.sh"
  fi
}

run_suites() {
  echo "==> Execution mode: $([[ "${USE_DOCKER}" == "1" ]] && echo docker || echo host)"
  echo "==> Ark host bundle: ${ARK_HOST_BUNDLE_DIR}"
  echo "==> Work root: ${WORK_ROOT}"
  if [[ "${USE_DOCKER}" == "1" ]]; then
    run_in_docker bash /work/scripts/ohos/arkvm_run_suites_in_docker.sh
  else
    run_on_host bash "${REPO_ROOT}/scripts/ohos/arkvm_run_suites_in_docker.sh"
  fi
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
build_host_libs
prepare_workspace
generate_suites
run_compile
run_suites
finalize
