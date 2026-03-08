#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." &>/dev/null && pwd)"

LEGACY_ARKVM_DIR="${ARKVM_X64_STATIC_DIR:-}"
ARK_HOST_TOOLS_DIR="${ARK_HOST_TOOLS_DIR:-${ARK_HYBRID_HOST_DIR:-${LEGACY_ARKVM_DIR:-/Users/ranger/Desktop/x64_linux_static}}}"
ARK_HOST_BUNDLE_DIR="${ARK_HOST_BUNDLE_DIR:-${ARK_HOST_TOOLS_DIR}}"
ARK_ES2ABC="${ARK_ES2ABC:-${ARK_HOST_BUNDLE_DIR}/es2abc}"
ARK_JS_NAPI_CLI="${ARK_JS_NAPI_CLI:-${ARK_HOST_BUNDLE_DIR}/ark_js_napi_cli}"
ARK_ACE_NAPI_LIB="${ARK_ACE_NAPI_LIB:-${ARK_HOST_BUNDLE_DIR}/libace_napi.so}"
ARK_STUB_FILE="${ARK_STUB_FILE:-${ARK_HOST_BUNDLE_DIR}/stub.an}"
ARK_HOST_CONTAINER_MOUNT="${ARK_HOST_CONTAINER_MOUNT:-/ark-host}"

DOCKER_IMAGE="${DOCKER_IMAGE:-docker.m.daocloud.io/library/ubuntu:latest}"
DOCKER_PLATFORM="${DOCKER_PLATFORM:-linux/amd64}"
TEST_TIMEOUT_SEC="${TEST_TIMEOUT_SEC:-90}"
KEEP_WORKDIR="${KEEP_WORKDIR:-0}"
FAIL_ON_CASE_ERROR="${FAIL_ON_CASE_ERROR:-1}"
EXAMPLE_BUILD_FEATURES="${EXAMPLE_BUILD_FEATURES:-arkvm-test}"
COMPACT_BUILD_FEATURES="${COMPACT_BUILD_FEATURES:-arkvm-test}"

ARK_HOST_CACHE_ROOT="${ARK_HOST_CACHE_ROOT:-${REPO_ROOT}/.cache/ark-host}"
TMPDIR="${ARK_HOST_TMPDIR:-${ARK_HOST_CACHE_ROOT}/tmp}"
WORK_ROOT="${ARK_HOST_WORK_ROOT:-${REPO_ROOT}/.tmp_ohos_split_runner}"
CONTAINER_WORK_ROOT="/work/.tmp_ohos_split_runner"
WORKSPACE_DIR="${WORK_ROOT}/workspace"
RESULT_FILE="${WORK_ROOT}/results.tsv"

mkdir -p "${TMPDIR}" "${ARK_HOST_CACHE_ROOT}"
export TMPDIR

if [[ ! -d "${ARK_HOST_BUNDLE_DIR}" ]]; then
  echo "Ark hybrid host bundle not found: ${ARK_HOST_BUNDLE_DIR}" >&2
  echo "Set ARK_HOST_TOOLS_DIR to a standalone hybrid host bundle directory." >&2
  exit 1
fi

for bin in "${ARK_ES2ABC}" "${ARK_JS_NAPI_CLI}"; do
  if [[ ! -x "${bin}" ]]; then
    echo "Missing binary: ${bin}" >&2
    exit 1
  fi
done

if [[ ! -f "${ARK_ACE_NAPI_LIB}" ]]; then
  echo "Missing shared lib: ${ARK_ACE_NAPI_LIB}" >&2
  exit 1
fi

ARK_STUB_ARGS=()
if [[ -f "${ARK_STUB_FILE}" ]]; then
  ARK_STUB_ARGS=(--stub-file "${ARK_STUB_FILE}")
else
  echo "==> Stub file not found, continue without --stub-file: ${ARK_STUB_FILE}"
fi

bash "${SCRIPT_DIR}/split_ohos_tests.sh"

if [[ "${SKIP_BUILD_LIBS:-0}" != "1" ]]; then
  docker run --rm \
    --platform "${DOCKER_PLATFORM}" \
    -v "${REPO_ROOT}:/work" \
    -v "${ARK_HOST_BUNDLE_DIR}:${ARK_HOST_CONTAINER_MOUNT}:ro" \
    -w /work \
    ohos-rs-arkvm-linux-x64:latest \
    /bin/bash -lc '
      set -euo pipefail
      export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
      export ARK_HOST_BUNDLE_DIR="'"${ARK_HOST_CONTAINER_MOUNT}"'"
      export ARK_ACE_NAPI_LIB="'"${ARK_HOST_CONTAINER_MOUNT}"'"/libace_napi.so
      cargo build -p example --release --features "'"${EXAMPLE_BUILD_FEATURES}"'"
      cargo build -p compact --release --features "'"${COMPACT_BUILD_FEATURES}"'"
    '
fi

rm -rf "${WORK_ROOT}"
mkdir -p "${WORKSPACE_DIR}" "${WORK_ROOT}/logs"
cp -R "${REPO_ROOT}/test" "${WORKSPACE_DIR}/"

echo "==> ArkJS + N-API host bundle: ${ARK_HOST_BUNDLE_DIR}"
echo "==> Work root: ${WORK_ROOT}"
echo "==> Stub args: ${ARK_STUB_ARGS[*]:-(none)}"

docker run --rm \
  --platform "${DOCKER_PLATFORM}" \
  -v "${REPO_ROOT}:/work" \
  -v "${ARK_HOST_BUNDLE_DIR}:${ARK_HOST_CONTAINER_MOUNT}:ro" \
  -w /work \
  "${DOCKER_IMAGE}" \
  /bin/bash -lc '
    set -euo pipefail
    WORK_ROOT="'"${CONTAINER_WORK_ROOT}"'"
    WORKSPACE_DIR="${WORK_ROOT}/workspace"
    RESULT_FILE="${WORK_ROOT}/results.tsv"
    TEST_TIMEOUT_SEC="'"${TEST_TIMEOUT_SEC}"'"
    HOST_MOUNT="'"${ARK_HOST_CONTAINER_MOUNT}"'"
    STUB_FILE="'"${ARK_STUB_FILE}"'"

    mkdir -p "${WORKSPACE_DIR}/module" "${WORK_ROOT}/logs" "${WORKSPACE_DIR}/test/ohos/src"
    cp /work/target/release/libexample.so "${WORKSPACE_DIR}/module/"
    cp /work/target/release/libcompact.so "${WORKSPACE_DIR}/module/"
    cp "${HOST_MOUNT}/libace_napi.so" "${WORKSPACE_DIR}/module/"
    cp /work/target/release/libexample.so "${WORKSPACE_DIR}/"
    cp /work/target/release/libcompact.so "${WORKSPACE_DIR}/"
    cp "${HOST_MOUNT}/libace_napi.so" "${WORKSPACE_DIR}/"
    cp /work/target/release/libexample.so "${WORKSPACE_DIR}/test/ohos/src/"
    cp /work/target/release/libcompact.so "${WORKSPACE_DIR}/test/ohos/src/"
    cp "${HOST_MOUNT}/libace_napi.so" "${WORKSPACE_DIR}/test/ohos/src/"

    : > "${RESULT_FILE}"
    echo -e "suite_id\tstatus\texit_code\tsuite_rel\trunner_status\texecuted\tpass\tfail\terror\tskip\tassertions\tnapi_calls\tissues" >> "${RESULT_FILE}"

    echo "==> Compile abc files for split test workspace"
    compile_failed=0
    while IFS= read -r -d "" src; do
      ext="${src##*.}"
      out="${src%.*}.abc"
      case "${ext}" in
        ts) parser_ext="ts" ;;
        ets) parser_ext="ts" ;;
        js) parser_ext="js" ;;
        *) continue ;;
      esac
      if ! "${HOST_MOUNT}/es2abc" --merge-abc --extension="${parser_ext}" --module --output "${out}" "${src}" \
        >"${WORK_ROOT}/logs/compile.log" 2>&1; then
        compile_failed=$((compile_failed + 1))
      fi
    done < <(find "${WORKSPACE_DIR}/test/ohos" -type f \( -name "*.ts" -o -name "*.ets" -o -name "*.js" \) -print0 | sort -z)

    echo "compile_failed=${compile_failed}" >> "${RESULT_FILE}"
    if [[ ${compile_failed} -gt 0 ]]; then
      exit 2
    fi

    total=0
    ok=0
    failed=0
    case_total=0
    case_pass=0
    case_fail=0
    case_error=0
    case_skip=0

    echo "==> Execute split suites one by one"
    while IFS="|" read -r suite_id suite_rel; do
      if [[ -z "${suite_id}" ]]; then
        continue
      fi

      total=$((total + 1))
      suite_abc="${WORKSPACE_DIR}/test/ohos/suites/${suite_id}.abc"
      log_file="${WORK_ROOT}/logs/${suite_id}.log"

      if [[ ! -f "${suite_abc}" ]]; then
        echo -e "${suite_id}\tmissing_abc\t127\t${suite_rel}" >> "${RESULT_FILE}"
        failed=$((failed + 1))
        continue
      fi

      set +e
      (
        cd "${WORKSPACE_DIR}"
        export LD_LIBRARY_PATH="${HOST_MOUNT}:/work/target/release:${WORKSPACE_DIR}:${WORKSPACE_DIR}/module:${WORKSPACE_DIR}/test/ohos/src"
        : > "${log_file}"
        cli_args=("${HOST_MOUNT}/ark_js_napi_cli")
        if [[ -f "${STUB_FILE}" ]]; then
          cli_args+=(--stub-file "${STUB_FILE}")
        fi
        cli_args+=(--entry-point "${suite_id}" "test/ohos/suites/${suite_id}.abc")
        "${cli_args[@]}" >"${log_file}" 2>&1 &
        suite_pid=$!
        deadline=$((SECONDS + TEST_TIMEOUT_SEC))

        while kill -0 "${suite_pid}" 2>/dev/null; do
          if grep -q "^__OHOS_SPLIT_RESULT__" "${log_file}" 2>/dev/null; then
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
      exit_code=$?
      set -e

      runner_marker="$(grep -E "^__OHOS_SPLIT_RESULT__" "${log_file}" | tail -n1 || true)"
      runner_status="$(echo "${runner_marker}" | sed -n "s/.* status=\\([^ ]*\\).*/\\1/p")"
      runner_executed="$(echo "${runner_marker}" | sed -n "s/.* executed=\\([0-9][0-9]*\\).*/\\1/p")"
      runner_total="$(echo "${runner_marker}" | sed -n "s/.* total=\\([0-9][0-9]*\\).*/\\1/p")"
      runner_pass="$(echo "${runner_marker}" | sed -n "s/.* pass=\\([0-9][0-9]*\\).*/\\1/p")"
      runner_failure="$(echo "${runner_marker}" | sed -n "s/.* failure=\\([0-9][0-9]*\\).*/\\1/p")"
      runner_error="$(echo "${runner_marker}" | sed -n "s/.* error=\\([0-9][0-9]*\\).*/\\1/p")"
      runner_ignore="$(echo "${runner_marker}" | sed -n "s/.* ignore=\\([0-9][0-9]*\\).*/\\1/p")"
      runner_assertions="$(echo "${runner_marker}" | sed -n "s/.* assertions=\\([0-9][0-9]*\\).*/\\1/p")"
      runner_napi_calls="$(echo "${runner_marker}" | sed -n "s/.* napi_calls=\\([0-9][0-9]*\\).*/\\1/p")"
      runner_issues="$(echo "${runner_marker}" | sed -n "s/.* issues=\\([^ ]*\\).*/\\1/p")"

      grep -E "^\\[RUN\\]|^  \\[(PASS|FAIL|ERROR|SKIP)\\]|^\\[SUMMARY\\]" "${log_file}" || true

      if [[ ${exit_code} -eq 124 ]]; then
        status="timeout"
        failed=$((failed + 1))
      elif [[ -z "${runner_marker}" ]]; then
        status="no_result_marker"
        failed=$((failed + 1))
      elif [[ -z "${runner_executed}" || "${runner_executed}" -le 0 ]]; then
        status="no_case_executed"
        failed=$((failed + 1))
      elif [[ "${runner_status}" == "ok" ]]; then
        status="ok"
        ok=$((ok + 1))
      else
        status="runner_fail"
        failed=$((failed + 1))
      fi

      if [[ -n "${runner_total}" ]]; then
        case_total=$((case_total + runner_total))
      fi
      if [[ -n "${runner_pass}" ]]; then
        case_pass=$((case_pass + runner_pass))
      fi
      if [[ -n "${runner_failure}" ]]; then
        case_fail=$((case_fail + runner_failure))
      fi
      if [[ -n "${runner_error}" ]]; then
        case_error=$((case_error + runner_error))
      fi
      if [[ -n "${runner_ignore}" ]]; then
        case_skip=$((case_skip + runner_ignore))
      fi

      echo -e "${suite_id}\t${status}\t${exit_code}\t${suite_rel}\t${runner_status:-missing}\t${runner_executed:-0}\t${runner_pass:-0}\t${runner_failure:-0}\t${runner_error:-0}\t${runner_ignore:-0}\t${runner_assertions:-0}\t${runner_napi_calls:-0}\t${runner_issues:-missing}" >> "${RESULT_FILE}"
      echo "[${total}] ${suite_id} -> ${status} (exit=${exit_code}, runner=${runner_status:-missing}, executed=${runner_executed:-0}, assertions=${runner_assertions:-0}, napi_calls=${runner_napi_calls:-0})"
    done < "/work/test/ohos/suites.list"

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

    echo
    echo "=== Suite Summary (PASS) ==="
    pass_suite_count="$(awk -F"\t" "NF >= 13 && \$1 != \"suite_id\" && \$2 == \"ok\" { c++ } END { print c + 0 }" "${RESULT_FILE}")"
    if [[ "${pass_suite_count}" -eq 0 ]]; then
      echo "(none)"
    else
      awk -F"\t" "NF >= 13 && \$1 != \"suite_id\" && \$2 == \"ok\" { printf \"[OK]   %s (pass=%s fail=%s error=%s skip=%s)\\n\", \$4, \$7, \$8, \$9, \$10 }" "${RESULT_FILE}"
    fi

    echo
    echo "=== Suite Summary (FAIL) ==="
    fail_suite_count="$(awk -F"\t" "NF >= 13 && \$1 != \"suite_id\" && \$2 != \"ok\" { c++ } END { print c + 0 }" "${RESULT_FILE}")"
    if [[ "${fail_suite_count}" -eq 0 ]]; then
      echo "(none)"
    else
      awk -F"\t" "NF >= 13 && \$1 != \"suite_id\" && \$2 != \"ok\" { printf \"[FAIL] %s status=%s issues=%s (pass=%s fail=%s error=%s skip=%s)\\n\", \$4, \$2, \$13, \$7, \$8, \$9, \$10 }" "${RESULT_FILE}"
    fi

    echo
    echo "=== Failed Cases ==="
    if [[ "${fail_suite_count}" -eq 0 ]]; then
      echo "(none)"
    else
      while IFS="$(printf "\t")" read -r suite_id suite_rel; do
        if [[ -z "${suite_id}" ]]; then
          continue
        fi
        log_file="${WORK_ROOT}/logs/${suite_id}.log"
        echo "-- ${suite_rel}"
        if [[ -f "${log_file}" ]]; then
          case_lines="$(grep -E "^  \[(FAIL|ERROR)\]" "${log_file}" || true)"
          if [[ -n "${case_lines}" ]]; then
            echo "${case_lines}" | sed "s/^  /    /"
          else
            echo "    (no FAIL/ERROR lines captured)"
          fi
        else
          echo "    (missing log file)"
        fi
      done < <(awk -F"\t" "NF >= 13 && \$1 != \"suite_id\" && \$2 != \"ok\" { printf \"%s\\t%s\\n\", \$1, \$4 }" "${RESULT_FILE}")
    fi
  '

echo "Results: ${RESULT_FILE}"
echo
cat "${RESULT_FILE}"

if [[ "${KEEP_WORKDIR}" != "1" ]]; then
  rm -rf "${WORKSPACE_DIR}"
fi

if [[ "${FAIL_ON_CASE_ERROR}" == "1" ]]; then
  failed_count="$(grep -E "^summary_failed=" "${RESULT_FILE}" | cut -d"=" -f2 | tail -n1)"
  if [[ "${failed_count:-0}" -gt 0 ]]; then
    exit 1
  fi
fi
