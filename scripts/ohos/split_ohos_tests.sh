#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." &>/dev/null && pwd)"

SOURCE_ROOT="${1:-${REPO_ROOT}/test/ohos/source}"
TARGET_ROOT="${2:-${REPO_ROOT}/test/ohos}"
DEFAULT_TMP_DIR="${DEFAULT_TMP_DIR:-/tmp}"
SUITES_ROOT="${TARGET_ROOT}/suites"
MANIFEST_FILE="${TARGET_ROOT}/suites.list"

if [[ ! -d "${SOURCE_ROOT}" ]]; then
  echo "Source test directory not found: ${SOURCE_ROOT}" >&2
  exit 1
fi

HAS_RG=0
if command -v rg >/dev/null 2>&1; then
  HAS_RG=1
fi

file_has_pattern() {
  local pattern="$1"
  local file="$2"
  if [[ "${HAS_RG}" == "1" ]]; then
    rg -q -- "${pattern}" "${file}"
    return $?
  fi
  LC_ALL=C LANG=C grep -E -q --binary-files=text -- "${pattern}" "${file}"
}

mkdir -p "${TARGET_ROOT}"
rm -rf "${SUITES_ROOT}"
mkdir -p "${SUITES_ROOT}"

: > "${MANIFEST_FILE}"

find "${SOURCE_ROOT}" -type f \( -name "*.test.ts" -o -name "*.test.ets" \) \
  ! -path "*/utils/*" \
  ! -name "Ability.test.ets" \
  ! -name "List.test.ets" | sort | while IFS= read -r suite_file; do
    if ! file_has_pattern "describe\\(" "${suite_file}"; then
      continue
    fi

    rel="${suite_file#${SOURCE_ROOT}/}"
    rel_no_ext="${rel%.*}"
    suite_id="$(echo "${rel_no_ext}" | sed -E 's#[^A-Za-z0-9]+#_#g; s#^_+##; s#_+$##')"
    entry_file="${SUITES_ROOT}/${suite_id}.ts"

    cat > "${entry_file}" <<SUITE_EOF
import "../runtime/console_shim";
import suite from "../source/${rel_no_ext}";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("${rel}", suite, "${DEFAULT_TMP_DIR}");
SUITE_EOF

    echo "${suite_id}|${rel}" >> "${MANIFEST_FILE}"
  done

count="$(wc -l < "${MANIFEST_FILE}" | tr -d '[:space:]')"
echo "Source root: ${SOURCE_ROOT}"
echo "Generated split suite entries: ${count}"
echo "Suite manifest: ${MANIFEST_FILE}"
