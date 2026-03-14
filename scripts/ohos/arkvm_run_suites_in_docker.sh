#!/usr/bin/env bash
set -euo pipefail

WORK_ROOT="${WORK_ROOT:?missing WORK_ROOT}"
WORKSPACE_DIR="${WORKSPACE_DIR:?missing WORKSPACE_DIR}"
TEST_ROOT="${TEST_ROOT:?missing TEST_ROOT}"
RESULT_FILE="${RESULT_FILE:?missing RESULT_FILE}"
HOST_MOUNT="${ARK_HOST_CONTAINER_MOUNT:?missing ARK_HOST_CONTAINER_MOUNT}"
TEST_TIMEOUT_SEC="${TEST_TIMEOUT_SEC:-90}"
STUB_FILE="${STUB_FILE:-}"
TARGET_RELEASE_DIR="${TARGET_RELEASE_DIR:-/work/target/arkvm-host/release}"
MANIFEST_FILE="${TEST_ROOT}/suites.list"

prepare_runtime() {
  mkdir -p "${WORKSPACE_DIR}/module" "${WORK_ROOT}/logs"

  printf '%s' 'Hello world!' > /tmp/test.txt
  cat > /tmp/test.json <<'JSON_EOF'
{
  "license": "MIT",
  "author": "richerfu",
  "name": "@ohos-rs/ada",
  "description": "ada binding for OpenHarmony, powered by ohos-rs",
  "main": "index.ets",
  "version": "0.0.2",
  "types": "libs/index.d.ts",
  "dependencies": {}
}
JSON_EOF

  cp "${TARGET_RELEASE_DIR}/libexample.so" "${WORKSPACE_DIR}/module/"
  cp "${TARGET_RELEASE_DIR}/libcompact.so" "${WORKSPACE_DIR}/module/"
  cp "${TARGET_RELEASE_DIR}/libbuffer.so" "${WORKSPACE_DIR}/module/"
  cp "${TARGET_RELEASE_DIR}/libworker.so" "${WORKSPACE_DIR}/module/"
  cp "${HOST_MOUNT}/libace_napi.so" "${WORKSPACE_DIR}/module/"
  cp "${HOST_MOUNT}/libets_interop_js_napi.so" "${WORKSPACE_DIR}/module/"

  cp "${TARGET_RELEASE_DIR}/libexample.so" "${WORKSPACE_DIR}/"
  cp "${TARGET_RELEASE_DIR}/libcompact.so" "${WORKSPACE_DIR}/"
  cp "${TARGET_RELEASE_DIR}/libbuffer.so" "${WORKSPACE_DIR}/"
  cp "${TARGET_RELEASE_DIR}/libworker.so" "${WORKSPACE_DIR}/"
  cp "${HOST_MOUNT}/libace_napi.so" "${WORKSPACE_DIR}/"
}

init_results() {
  : > "${RESULT_FILE}"
  printf 'suite_id\tstatus\texit_code\tsuite_rel\trunner_status\texecuted\tpass\tfail\terror\tskip\tassertions\tnapi_calls\tissues\n' >> "${RESULT_FILE}"
  echo 'compile_failed=0' >> "${RESULT_FILE}"
}

extract_field() {
  local key="$1"
  local marker="$2"
  local part
  for part in ${marker}; do
    if [[ "${part}" == "${key}="* ]]; then
      echo "${part#${key}=}"
      return 0
    fi
  done
}

run_suite_once() {
  local suite_id="$1"
  local suite_abc_rel="$2"
  local log_file="$3"
  local suite_pid deadline

  (
    cd "${WORKSPACE_DIR}"
    export LD_LIBRARY_PATH="${HOST_MOUNT}:${TARGET_RELEASE_DIR}:${WORKSPACE_DIR}:${WORKSPACE_DIR}/module"
    export ARK_JS_NAPI_CLI="${HOST_MOUNT}/ark_js_napi_cli"
    export ARK_ES2ABC="${HOST_MOUNT}/es2abc"
    export ARK_STUB_FILE="${STUB_FILE}"
    export OHOS_RS_WORKSPACE_DIR="${WORKSPACE_DIR}"
    export OHOS_RS_WORKER_BOOTSTRAP_ABC="${WORKSPACE_DIR}/third_party/openharmony/worker/runtime/bootstrap.abc"

    : > "${log_file}"
    cli_args=("${HOST_MOUNT}/ark_js_napi_cli")
    [[ -f "${STUB_FILE}" ]] && cli_args+=(--stub-file "${STUB_FILE}")
    cli_args+=(--entry-point "${suite_id}" "${suite_abc_rel}")
    "${cli_args[@]}" >"${log_file}" 2>&1 &
    suite_pid=$!
    deadline=$((SECONDS + TEST_TIMEOUT_SEC))

    while kill -0 "${suite_pid}" 2>/dev/null; do
      if grep -q '^__OHOS_SPLIT_RESULT__' "${log_file}" 2>/dev/null; then
        kill -TERM "${suite_pid}" 2>/dev/null || true
        sleep 1
        kill -KILL "${suite_pid}" 2>/dev/null || true
        wait "${suite_pid}" >/dev/null 2>&1 || true
        exit 0
      fi
      if (( SECONDS >= deadline )); then
        kill -TERM "${suite_pid}" 2>/dev/null || true
        sleep 1
        kill -KILL "${suite_pid}" 2>/dev/null || true
        wait "${suite_pid}" >/dev/null 2>&1 || true
        exit 124
      fi
      sleep 0.2
    done

    wait "${suite_pid}"
  )
}

prepare_runtime
init_results

declare -i total=0 ok=0 failed=0 case_total=0 case_pass=0 case_fail=0 case_error=0 case_skip=0
while IFS='|' read -r suite_id suite_rel; do
  [[ -z "${suite_id}" ]] && continue

  total+=1
  suite_abc_abs="${TEST_ROOT}/suites/${suite_id}.abc"
  suite_abc_rel="test/ohos/suites/${suite_id}.abc"
  log_file="${WORK_ROOT}/logs/${suite_id}.log"

  if [[ ! -f "${suite_abc_abs}" ]]; then
    printf '%s\tmissing_abc\t127\t%s\tmissing\t0\t0\t0\t0\t0\t0\t0\tmissing\n' "${suite_id}" "${suite_rel}" >> "${RESULT_FILE}"
    failed+=1
    continue
  fi

  set +e
  run_suite_once "${suite_id}" "${suite_abc_rel}" "${log_file}"
  exit_code=$?
  set -e

  marker="$(grep -E '^__OHOS_SPLIT_RESULT__' "${log_file}" | tail -n1 || true)"
  runner_status="$(extract_field status "${marker}")"
  executed="$(extract_field executed "${marker}")"
  suite_total="$(extract_field total "${marker}")"
  suite_pass="$(extract_field pass "${marker}")"
  suite_fail="$(extract_field failure "${marker}")"
  suite_error="$(extract_field error "${marker}")"
  suite_skip="$(extract_field ignore "${marker}")"
  assertions="$(extract_field assertions "${marker}")"
  napi_calls="$(extract_field napi_calls "${marker}")"
  issues="$(extract_field issues "${marker}")"

  grep -E '^\[RUN\]|^  \[(PASS|FAIL|ERROR|SKIP)\]|^\[SUMMARY\]' "${log_file}" || true

  if [[ "${exit_code}" -eq 124 ]]; then
    status='timeout'
    failed+=1
  elif [[ -z "${marker}" ]]; then
    status='no_result_marker'
    failed+=1
  elif [[ -z "${executed}" || "${executed}" -le 0 ]]; then
    status='no_case_executed'
    failed+=1
  elif [[ "${runner_status}" == 'ok' ]]; then
    status='ok'
    ok+=1
  else
    status='runner_fail'
    failed+=1
  fi

  [[ -n "${suite_total}" ]] && case_total=$((case_total + suite_total))
  [[ -n "${suite_pass}" ]] && case_pass=$((case_pass + suite_pass))
  [[ -n "${suite_fail}" ]] && case_fail=$((case_fail + suite_fail))
  [[ -n "${suite_error}" ]] && case_error=$((case_error + suite_error))
  [[ -n "${suite_skip}" ]] && case_skip=$((case_skip + suite_skip))

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${suite_id}" "${status}" "${exit_code}" "${suite_rel}" "${runner_status:-missing}" \
    "${executed:-0}" "${suite_pass:-0}" "${suite_fail:-0}" "${suite_error:-0}" \
    "${suite_skip:-0}" "${assertions:-0}" "${napi_calls:-0}" "${issues:-missing}" >> "${RESULT_FILE}"

  echo "[${total}] ${suite_id} -> ${status}"
done < "${MANIFEST_FILE}"

echo "summary_total=${total}" >> "${RESULT_FILE}"
echo "summary_ok=${ok}" >> "${RESULT_FILE}"
echo "summary_failed=${failed}" >> "${RESULT_FILE}"
echo "case_total=${case_total}" >> "${RESULT_FILE}"
echo "case_pass=${case_pass}" >> "${RESULT_FILE}"
echo "case_fail=${case_fail}" >> "${RESULT_FILE}"
echo "case_error=${case_error}" >> "${RESULT_FILE}"
echo "case_skip=${case_skip}" >> "${RESULT_FILE}"

echo "Suite summary: total=${total}, pass=${ok}, fail=${failed}"
echo "Case summary: total=${case_total}, pass=${case_pass}, fail=${case_fail}, error=${case_error}, skip=${case_skip}"
