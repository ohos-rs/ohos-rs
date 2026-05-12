#!/usr/bin/env bash
set -euo pipefail

WORK_ROOT="${WORK_ROOT:?missing WORK_ROOT}"
WORKSPACE_DIR="${WORKSPACE_DIR:?missing WORKSPACE_DIR}"
TEST_ROOT="${TEST_ROOT:?missing TEST_ROOT}"
ARK_HOST_BUNDLE_DIR="${ARK_HOST_BUNDLE_DIR:?missing ARK_HOST_BUNDLE_DIR}"
COMPILE_LOG="${WORK_ROOT}/logs/compile.log"

mkdir -p "${WORK_ROOT}/logs"
: > "${COMPILE_LOG}"

declare -i compile_failed=0
while IFS= read -r -d '' src; do
  ext="${src##*.}"
  out="${src%.*}.abc"
  case "${ext}" in
    ts|ets) parser_ext="ts" ;;
    js) parser_ext="js" ;;
    *) continue ;;
  esac

  if ! "${ARK_HOST_BUNDLE_DIR}/es2abc" --merge-abc --extension="${parser_ext}" --module --output "${out}" "${src}" >>"${COMPILE_LOG}" 2>&1; then
    compile_failed+=1
  fi
done < <({
  find "${TEST_ROOT}" -type f \( -name '*.ts' -o -name '*.ets' -o -name '*.js' \) -print0
  if [[ -d "${WORKSPACE_DIR}/third_party" ]]; then
    find "${WORKSPACE_DIR}/third_party" -type f \( -name '*.ts' -o -name '*.ets' -o -name '*.js' \) -print0
  fi
} | sort -z)

if (( compile_failed > 0 )); then
  echo "compile_failed=${compile_failed}" >&2
  exit 2
fi
