#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." &>/dev/null && pwd)"

SOURCE_ROOT="${1:-${REPO_ROOT}/test/ohos/source}"
TARGET_ROOT="${2:-${REPO_ROOT}/test/ohos}"
DEFAULT_TMP_DIR="${DEFAULT_TMP_DIR:-/tmp}"

SRC_COPY_ROOT="${TARGET_ROOT}/src"
SUITES_ROOT="${TARGET_ROOT}/suites"
RUNTIME_ROOT="${TARGET_ROOT}/runtime"
SHIMS_ROOT="${SRC_COPY_ROOT}/utils/shims"
MANIFEST_FILE="${TARGET_ROOT}/suites.list"

if [[ ! -d "${SOURCE_ROOT}" ]]; then
  echo "Source test directory not found: ${SOURCE_ROOT}" >&2
  exit 1
fi

HAS_RG=0
if command -v rg >/dev/null 2>&1; then
  HAS_RG=1
fi

rel_path() {
  LC_ALL=C LANG=C perl -MFile::Spec -MCwd=abs_path \
    -e 'print File::Spec->abs2rel(abs_path($ARGV[0]), abs_path($ARGV[1]))' \
    "$1" "$2"
}

list_matching_files() {
  local pattern="$1"
  local root="$2"
  shift 2
  local exts=("$@")

  if [[ "${HAS_RG}" == "1" ]]; then
    local cmd=(rg -a -l)
    local ext
    for ext in "${exts[@]}"; do
      cmd+=(-g "*.${ext}")
    done
    cmd+=(-- "${pattern}" "${root}")
    "${cmd[@]}" || true
    return 0
  fi

  local find_expr=()
  local first=1
  local ext
  for ext in "${exts[@]}"; do
    if [[ "${first}" == "0" ]]; then
      find_expr+=(-o)
    fi
    find_expr+=(-name "*.${ext}")
    first=0
  done

  while IFS= read -r -d '' file; do
    if LC_ALL=C LANG=C grep -E -q --binary-files=text -- "${pattern}" "${file}"; then
      printf '%s\n' "${file}"
    fi
  done < <(find "${root}" -type f \( "${find_expr[@]}" \) -print0)
}

file_has_pattern() {
  local pattern="$1"
  local file="$2"
  if [[ "${HAS_RG}" == "1" ]]; then
    rg -q -- "${pattern}" "${file}"
    return $?
  fi
  LC_ALL=C LANG=C grep -E -q --binary-files=text -- "${pattern}" "${file}"
}

rewrite_module_imports() {
  local module_name="$1"
  local shim_file="$2"

  list_matching_files "${module_name}" "${SRC_COPY_ROOT}" ts ets js | while IFS= read -r file; do
      local file_dir
      file_dir="$(cd -- "$(dirname -- "${file}")" &>/dev/null && pwd)"
      local rel
      rel="$(rel_path "${shim_file}" "${file_dir}")"
      rel="${rel%.ts}"
      rel="${rel%.ets}"
      rel="${rel%.js}"
      if [[ "${rel}" != ./* && "${rel}" != ../* ]]; then
        rel="./${rel}"
      fi
      MODULE_NAME="${module_name}" REPLACEMENT="${rel}" LC_ALL=C LANG=C perl -0777 -i -pe '
        $module = quotemeta($ENV{"MODULE_NAME"});
        $replacement = $ENV{"REPLACEMENT"};
        s/"$module"/"$replacement"/g;
      ' "${file}"
    done || true
}

mkdir -p "${TARGET_ROOT}" "${RUNTIME_ROOT}"
rm -rf "${SRC_COPY_ROOT}" "${SUITES_ROOT}"
mkdir -p "${SRC_COPY_ROOT}" "${SUITES_ROOT}" "${SHIMS_ROOT}"

cp -R "${SOURCE_ROOT}/." "${SRC_COPY_ROOT}/"
rm -rf "${SRC_COPY_ROOT}/split"

if [[ -f "${SRC_COPY_ROOT}/utils/rx/index.test.ts" ]]; then
  mv "${SRC_COPY_ROOT}/utils/rx/index.test.ts" "${SRC_COPY_ROOT}/utils/rx/rx_index.test.ts"
fi

if [[ -f "${SRC_COPY_ROOT}/utils/core/service.js" ]]; then
  LC_ALL=C LANG=C perl -0777 -i -pe '
    s/async asyncRun\(coreContext\) \{\n\s*const dataDriver/async asyncRun(coreContext) {\n    this.isExecuted = true;\n    const dataDriver/s;
  ' "${SRC_COPY_ROOT}/utils/core/service.js"
fi

list_matching_files "utils/rx/index\\.test" "${SRC_COPY_ROOT}" ts ets | while IFS= read -r file; do
  LC_ALL=C LANG=C perl -0777 -i -pe 's#utils/rx/index\.test#utils/rx/rx_index.test#g' "${file}"
done || true

cat > "${SHIMS_ROOT}/arkts.test.ts" <<'EOF'
function getNative(): ESObject {
  return requireNapiPreview("example", true);
}

function encodeUtf8(input: string): Uint8Array {
  const out = [];
  for (let i = 0; i < input.length; i++) {
    const code = input.charCodeAt(i);
    if (code <= 0x7f) {
      out.push(code);
      continue;
    }
    out.push(0x3f);
  }
  return new Uint8Array(out);
}

function decodeUtf8(input: Uint8Array): string {
  let out = "";
  for (const item of input) {
    out += String.fromCharCode(item);
  }
  return out;
}

function wrapBuffer(data: Uint8Array) {
  const value = data as ESObject;
  value.toString = (encoding: string = "utf8") => {
    if (encoding !== "utf8") {
      return decodeUtf8(data);
    }
    return decodeUtf8(data);
  };
  return value;
}

export const buffer = {
  from(data: ESObject) {
    if (typeof data === "string") {
      return wrapBuffer(encodeUtf8(data));
    }
    if (data instanceof ArrayBuffer) {
      return wrapBuffer(new Uint8Array(data));
    }
    if (Array.isArray(data)) {
      return wrapBuffer(new Uint8Array(data));
    }
    if (data instanceof Uint8Array) {
      return wrapBuffer(data);
    }
    return wrapBuffer(new Uint8Array(0));
  },
  alloc(size: number) {
    return wrapBuffer(new Uint8Array(size));
  },
  isBuffer(value: ESObject) {
    return value instanceof Uint8Array || value instanceof ArrayBuffer;
  },
  concat(chunks: Array<Uint8Array>) {
    const total = chunks.reduce((sum, item) => sum + item.length, 0);
    const out = new Uint8Array(total);
    let offset = 0;
    for (const item of chunks) {
      out.set(item, offset);
      offset += item.length;
    }
    return wrapBuffer(out);
  },
};

class ThreadWorker {
  private _onmessage: ((msg: ESObject) => void) | null = null;
  private _onerror: ((err: ESObject) => void) | null = null;
  private _onmessageerror: ((err: ESObject) => void) | null = null;
  private pendingMessages: Array<ESObject> = [];
  name: string;

  constructor(scriptURL: string, options?: ESObject) {
    this.name = options?.name || scriptURL || "mock-worker";
  }

  get onmessage() {
    return this._onmessage;
  }

  set onmessage(handler: ((msg: ESObject) => void) | null) {
    this._onmessage = handler;
    this.flushPendingMessages();
  }

  get onerror() {
    return this._onerror;
  }

  set onerror(handler: ((err: ESObject) => void) | null) {
    this._onerror = handler;
  }

  get onmessageerror() {
    return this._onmessageerror;
  }

  set onmessageerror(handler: ((err: ESObject) => void) | null) {
    this._onmessageerror = handler;
  }

  private flushPendingMessages() {
    if (!this._onmessage || this.pendingMessages.length === 0) {
      return;
    }
    const pending = this.pendingMessages.slice();
    this.pendingMessages = [];
    for (const payload of pending) {
      this._onmessage(payload);
    }
  }

  private emitMessage(data: ESObject) {
    const payload = { data, currentThreadName: this.name };
    if (this._onmessage) {
      this._onmessage(payload);
      return;
    }
    this.pendingMessages.push(payload);
  }

  private emitError(error: ESObject) {
    if (this._onerror) {
      this._onerror(error);
      return;
    }
    if (this._onmessageerror) {
      this._onmessageerror(error);
    }
  }

  private handleMessage(data: ESObject) {
    switch (data?.type) {
      case "require":
        return getNative().Animal.withKind(getNative().Kind.Cat).whoami() + getNative().DEFAULT_COST;
      case "async:buffer":
        return Promise.all(
          Array.from({ length: 100 }).map(() => getNative().bufferPassThrough(buffer.from([1, 2, 3]).buffer)),
        ).then(() => "done");
      case "async:arraybuffer":
        return Promise.all(
          Array.from({ length: 100 }).map(() => getNative().arrayBufferPassThrough(Uint8Array.from([1, 2, 3]))),
        ).then(() => "done");
      case "constructor": {
        let ellie = null;
        for (let i = 0; i < 1000; i++) {
          ellie = new getNative().Animal(getNative().Kind.Cat, "Ellie");
        }
        return ellie.name;
      }
      default:
        throw new TypeError(`Unknown message type: ${data?.type}`);
    }
  }

  postMessage(data: ESObject) {
    try {
      const result = this.handleMessage(data);
      if (result && typeof (result as ESObject).then === "function") {
        (result as Promise<ESObject>)
          .then((value) => {
            this.emitMessage(value);
          })
          .catch((error) => {
            this.emitError(error);
          });
        return;
      }
      this.emitMessage(result);
    } catch (error) {
      this.emitError(error);
    }
  }

  terminate() {
    this.pendingMessages = [];
    return Promise.resolve();
  }
}

export const worker = {
  ThreadWorker,
};
EOF

cat > "${SHIMS_ROOT}/ohos-worker.test.ts" <<'EOF'
import { worker } from "./arkts.test";

export default worker;
EOF

cat > "${SHIMS_ROOT}/emitter.test.ts" <<'EOF'
const handlers = new Map<string, Array<ESObject>>();

function on(name: string, handler: ESObject) {
  const list = handlers.get(name) || [];
  list.push(handler);
  handlers.set(name, list);
}

function off(name: string, handler: ESObject) {
  const list = handlers.get(name) || [];
  handlers.set(
    name,
    list.filter((item) => item !== handler),
  );
}

function emit(name: string, payload: ESObject) {
  const list = handlers.get(name) || [];
  for (const handler of list) {
    handler(payload);
  }
}

export default {
  on,
  off,
  emit,
};
EOF

cat > "${SHIMS_ROOT}/corefilekit.test.ts" <<'EOF'
const inMemoryFiles = new Map<string, string>();

export const fileIo = {
  AccessModeType: {
    EXIST: 0,
  },
  OpenMode: {
    READ_WRITE: 1,
    CREATE: 2,
  },
  accessSync(path: string) {
    return inMemoryFiles.has(path);
  },
  unlinkSync(path: string) {
    inMemoryFiles.delete(path);
  },
  openSync(path: string) {
    if (!inMemoryFiles.has(path)) {
      inMemoryFiles.set(path, "");
    }
    return { fd: path };
  },
  writeSync(fd: string, content: ESObject) {
    const current = inMemoryFiles.get(fd) || "";
    inMemoryFiles.set(fd, current + String(content));
  },
};
EOF

cat > "${SHIMS_ROOT}/filefs.test.js" <<'EOF'
const inMemoryFiles = new Map();

const fs = {
  OpenMode: {
    READ_WRITE: 1,
    CREATE: 2,
  },
  accessSync(path) {
    return inMemoryFiles.has(path);
  },
  unlinkSync(path) {
    inMemoryFiles.delete(path);
  },
  mkdirSync(_path) {},
  openSync(path) {
    if (!inMemoryFiles.has(path)) {
      inMemoryFiles.set(path, "");
    }
    return { fd: path };
  },
  writeSync(fd, content) {
    const current = inMemoryFiles.get(fd) || "";
    inMemoryFiles.set(fd, current + String(content));
    return String(content).length;
  },
  closeSync(_file) {},
};

export default fs;
EOF

cat > "${RUNTIME_ROOT}/no_ability_runner.ts" <<'EOF'
import { Hypium } from "../src/utils/framework.test";
import Core from "../src/utils/core/core";

type SplitMetrics = {
  registered: number;
  executed: number;
  assertionCalls: number;
  noAssertionTests: number;
  currentTest: string;
};

type NapiMetrics = {
  calls: number;
  modules: string[];
};

type CaseResult = {
  name: string;
  status: string;
  duration: number;
  message: string;
  stack: string;
};

function countExecutedSpecs(suite: ESObject): ESObject {
  let total = 0;
  let executed = 0;
  let skipped = 0;

  if (suite.specs && Array.isArray(suite.specs)) {
    total += suite.specs.length;
    for (const spec of suite.specs) {
      if (spec && spec.isSkip) {
        skipped += 1;
      }
      if (spec && spec.isExecuted) {
        executed += 1;
      }
    }
  }

  if (suite.childSuites && Array.isArray(suite.childSuites)) {
    for (const child of suite.childSuites) {
      const childStat = countExecutedSpecs(child);
      total += childStat.total;
      executed += childStat.executed;
      skipped += childStat.skipped;
    }
  }

  return { total, executed, skipped };
}

function getSplitMetrics(): SplitMetrics {
  const g = globalThis as ESObject;
  if (g.__ohosSplitMetrics__) {
    return g.__ohosSplitMetrics__ as SplitMetrics;
  }
  return {
    registered: 0,
    executed: 0,
    assertionCalls: 0,
    noAssertionTests: 0,
    currentTest: "",
  };
}

function getNapiMetrics(): NapiMetrics {
  const g = globalThis as ESObject;
  if (g.__ohosNapiMetrics__) {
    return g.__ohosNapiMetrics__ as NapiMetrics;
  }
  return {
    calls: 0,
    modules: [],
  };
}

function emitResultMarker(result: ESObject) {
  const issuesText =
    result.issues && Array.isArray(result.issues) && result.issues.length > 0
      ? result.issues.join(",")
      : "none";
  print(
    "__OHOS_SPLIT_RESULT__" +
      ` suite=${result.suite}` +
      ` status=${result.status}` +
      ` registered=${result.registered}` +
      ` executed=${result.executed}` +
      ` skipped=${result.skipped}` +
      ` total=${result.total}` +
      ` pass=${result.pass}` +
      ` failure=${result.failure}` +
      ` error=${result.error}` +
      ` ignore=${result.ignore}` +
      ` assertions=${result.assertions}` +
      ` no_assert_tests=${result.noAssertTests}` +
      ` napi_calls=${result.napiCalls}` +
      ` strict=${result.strictAssert}` +
      ` issues=${issuesText}`,
  );
}

function toText(value: ESObject): string {
  if (value === null || value === undefined) {
    return "";
  }
  if (typeof value === "string") {
    return value;
  }
  return `${value}`;
}

function getSetupErrorState(): ESObject | null {
  const g = globalThis as ESObject;
  return g.__ohosSplitSetupError__ || null;
}

function setSetupErrorState(error: ESObject) {
  const g = globalThis as ESObject;
  g.__ohosSplitSetupError__ = {
    message: toText(error && (error.message || error)),
    stack: toText(error && error.stack),
  };
}

function stackPreview(stackText: string, maxLines: number = 5): string[] {
  if (!stackText) {
    return [];
  }
  return stackText
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .slice(0, maxLines);
}

function printSetupError(setupError: ESObject) {
  if (!setupError) {
    return;
  }
  print("  [ERROR] <setup> (0ms)");
  const message = toText(setupError.message);
  if (message.length > 0) {
    print(`         ${message}`);
  }
  for (const line of stackPreview(toText(setupError.stack))) {
    print(`         ${line}`);
  }
}

function collectCaseResults(suite: ESObject, path: string[], out: CaseResult[]) {
  const nextPath = path.slice();
  const desc = toText(suite && suite.description);
  if (desc.length > 0) {
    nextPath.push(desc);
  }

  if (suite && suite.specs && Array.isArray(suite.specs)) {
    for (const spec of suite.specs) {
      const specName = toText(spec && spec.description) || "<unknown>";
      const fullName =
        nextPath.length > 0 ? `${nextPath.join(" > ")} > ${specName}` : specName;

      let status = "pass";
      let message = "";
      let stack = "";
      if (spec && spec.isSkip) {
        status = "skip";
      } else if (spec && spec.error) {
        status = "error";
        message = toText(spec.error.message || spec.error);
        stack = toText(spec.error.stack);
      } else if (spec && spec.fail) {
        status = "fail";
        message = toText(spec.fail.message || spec.fail);
        stack = toText(spec.fail.stack);
      } else if (!(spec && spec.pass === true)) {
        status = "unknown";
      }

      out.push({
        name: fullName,
        status,
        duration: Number((spec && spec.duration) || 0),
        message,
        stack,
      });
    }
  }

  if (suite && suite.childSuites && Array.isArray(suite.childSuites)) {
    for (const child of suite.childSuites) {
      collectCaseResults(child, nextPath, out);
    }
  }
}

function summarizeCases(cases: CaseResult[]): ESObject {
  let pass = 0;
  let fail = 0;
  let error = 0;
  let skip = 0;
  let unknown = 0;
  for (const item of cases) {
    if (item.status === "pass") {
      pass += 1;
    } else if (item.status === "fail") {
      fail += 1;
    } else if (item.status === "error") {
      error += 1;
    } else if (item.status === "skip") {
      skip += 1;
    } else {
      unknown += 1;
    }
  }
  return {
    total: cases.length,
    pass,
    fail,
    error,
    skip,
    unknown,
  };
}

function printCaseReport(suiteName: string, cases: CaseResult[], duration: number) {
  print(`[RUN] ${suiteName}`);
  for (const item of cases) {
    if (item.status === "pass") {
      print(`  [PASS] ${item.name} (${item.duration}ms)`);
      continue;
    }
    if (item.status === "skip") {
      print(`  [SKIP] ${item.name}`);
      continue;
    }
    const tag = item.status === "error" ? "ERROR" : "FAIL";
    print(`  [${tag}] ${item.name} (${item.duration}ms)`);
    if (item.message.length > 0) {
      print(`         ${item.message}`);
    }
    for (const line of stackPreview(item.stack)) {
      print(`         ${line}`);
    }
  }

  const summary = summarizeCases(cases);
  print(
    `[SUMMARY] suite=${suiteName} total=${summary.total} pass=${summary.pass} fail=${summary.fail} error=${summary.error} skip=${summary.skip} unknown=${summary.unknown} duration=${duration}ms`,
  );
}

function buildDoneState(suiteName: string, message: string, code: number, extraIssues: string[]): ESObject {
  const core = Core.getInstance();
  const suiteService = core.getDefaultService("suite");
  const rootSuite = suiteService ? suiteService.rootSuite : null;
  const summary = suiteService
    ? suiteService.getSummary()
    : { total: 0, pass: 0, failure: 0, error: 0, ignore: 0, duration: 0 };
  const specStat = rootSuite ? countExecutedSpecs(rootSuite) : { total: 0, executed: 0, skipped: 0 };
  const metrics = getSplitMetrics();
  const napiMetrics = getNapiMetrics();
  const strictAssert = (globalThis as ESObject).__ohosAssertStrict__ !== false;
  const setupError = getSetupErrorState();
  const issues = extraIssues.slice();
  const caseResults: CaseResult[] = [];

  if (rootSuite) {
    collectCaseResults(rootSuite, [], caseResults);
  }
  printCaseReport(suiteName, caseResults, Number(summary.duration || 0));
  if (setupError) {
    printSetupError(setupError);
    issues.push("setup_error");
  }
  const caseSummary = summarizeCases(caseResults);

  if (specStat.executed <= 0) {
    issues.push("no_callbacks");
  }
  if (specStat.executed + specStat.skipped !== specStat.total) {
    issues.push("spec_count_mismatch");
  }
  if (caseSummary.fail > 0 || caseSummary.error > 0 || caseSummary.unknown > 0) {
    issues.push("case_failures");
  }
  if (strictAssert && napiMetrics.modules.length > 0 && napiMetrics.calls <= 0) {
    issues.push("no_napi_calls");
  }

  return {
    suite: suiteName,
    message,
    code,
    registered: specStat.total,
    executed: specStat.executed,
    skipped: specStat.skipped,
    total: summary.total,
    pass: summary.pass,
    failure: summary.failure,
    error: summary.error,
    ignore: summary.ignore,
    assertions: metrics.assertionCalls,
    noAssertTests: metrics.noAssertionTests,
    napiCalls: napiMetrics.calls,
    napiModules: napiMetrics.modules.join(","),
    strictAssert,
    status: issues.length === 0 ? "ok" : "fail",
    issues,
  };
}

function emitFallbackResult(suiteName: string, reason: string) {
  const g = globalThis as ESObject;
  if (g.__ohosSplitDone__) {
    return;
  }
  const setupError = getSetupErrorState();
  const message = setupError ? toText(setupError.message) : reason;
  const doneState = buildDoneState(suiteName, message, 1, [reason]);
  emitResultMarker(doneState as ESObject);
  g.__ohosSplitDone__ = doneState;
}

class CliDelegator {
  printSync(_message: string) {
  }

  async print(_message: string) {
  }

  finishTest(message: string, code: number, callback: ESObject) {
    const g = globalThis as ESObject;
    if (g.__ohosSplitDone__) {
      if (callback) {
        callback();
      }
      return;
    }

    const suiteName = String(g.__ohosSplitSuiteName__ || "unknown");
    const doneState = buildDoneState(suiteName, message, code, []);

    emitResultMarker(doneState as ESObject);
    g.__ohosSplitDone__ = doneState;
    if (callback) {
      callback();
    }
  }

  executeShellCommand(_cmd: string, _timeout: number, callback: ESObject) {
    callback(null, { stdResult: "0" });
  }
}

export function runSplitSuite(suiteName: string, suite: ESObject, tmpDir: string = "/tmp") {
  const g = globalThis as ESObject;
  g.__ohosSplitSuiteName__ = suiteName;
  g.__ohosSplitDone__ = null;
  g.__ohosSplitSetupError__ = null;
  g.__ohosSplitImmediateExecute__ = true;
  g.__ohosSplitMetrics__ = {
    registered: 0,
    executed: 0,
    assertionCalls: 0,
    noAssertionTests: 0,
    currentTest: "",
  };
  print(`__OHOS_SPLIT_START__ suite=${suiteName}`);
  const delegator = new CliDelegator();
  const args = { parameters: {} };
  const ctx = { tempDir: tmpDir };

  try {
    const execution = Hypium.hypiumTest(delegator as ESObject, args as ESObject, ctx as ESObject, () => {
      try {
        suite(tmpDir);
      } catch (error) {
        setSetupErrorState(error);
      }
    });

    if (execution && typeof (execution as ESObject).then === "function") {
      (execution as Promise<ESObject>)
        .then(() => {
          emitFallbackResult(suiteName, getSetupErrorState() ? "setup_throw" : "finish_not_called");
        })
        .catch((error) => {
          setSetupErrorState(error);
          emitFallbackResult(suiteName, "execution_throw");
        });
      return;
    }

    emitFallbackResult(suiteName, getSetupErrorState() ? "setup_throw" : "finish_not_called");
  } catch (error) {
    setSetupErrorState(error);
    emitFallbackResult(suiteName, "bootstrap_throw");
  }
}
EOF

cat > "${RUNTIME_ROOT}/console_shim.ts" <<'EOF'
if (!(globalThis as ESObject).console) {
  (globalThis as ESObject).console = {
    log: (_msg: ESObject) => {},
    info: (_msg: ESObject) => {},
    warn: (_msg: ESObject) => {},
    error: (_msg: ESObject) => {},
  };
}

if (!(globalThis as ESObject).setTimeout) {
  let nextTimerId = 1;
  const timers = new Map<number, boolean>();
  (globalThis as ESObject).setTimeout = (handler: ESObject, timeoutMs: number = 0) => {
    const id = nextTimerId++;
    timers.set(id, true);
    if (typeof handler === "function") {
      Promise.resolve().then(() => {
        if (!timers.has(id)) {
          return;
        }
        timers.delete(id);
        handler();
      });
    }
    return id;
  };
  (globalThis as ESObject).__ohosTimerMap__ = timers;
}

if (!(globalThis as ESObject).clearTimeout) {
  (globalThis as ESObject).clearTimeout = (id: number) => {
    const timers = (globalThis as ESObject).__ohosTimerMap__;
    if (timers && typeof timers.delete === "function") {
      timers.delete(id);
    }
  };
}

function getOrInitNapiMetrics(): ESObject {
  const g = globalThis as ESObject;
  if (!g.__ohosNapiMetrics__) {
    g.__ohosNapiMetrics__ = {
      calls: 0,
      modules: [],
    };
  }
  return g.__ohosNapiMetrics__;
}

const originalRequireNapiPreview = (globalThis as ESObject).requireNapiPreview;
if (typeof originalRequireNapiPreview === "function") {
  const proxyCache = new WeakMap();
  const rawValueCache = new WeakMap();

  const unwrapValue = (value: ESObject): ESObject => {
    if (value === null || value === undefined) {
      return value;
    }
    const valueType = typeof value;
    if ((valueType === "object" || valueType === "function") && rawValueCache.has(value)) {
      return rawValueCache.get(value);
    }
    return value;
  };

  const wrapValue = (value: ESObject): ESObject => {

    if (value === null || value === undefined) {
      return value;
    }
    const valueType = typeof value;
    if (valueType !== "object" && valueType !== "function") {
      return value;
    }
    if (proxyCache.has(value)) {
      return proxyCache.get(value);
    }

    const proxy = new Proxy(value as ESObject, {
      get(target: ESObject, prop: ESObject, receiver: ESObject) {
        const inner = Reflect.get(target, prop, target);
        return wrapValue(inner);
      },
      set(target: ESObject, prop: ESObject, value: ESObject, receiver: ESObject) {
        return Reflect.set(target, prop, unwrapValue(value), target);
      },
      apply(target: ESObject, thisArg: ESObject, argArray: ESObject[]) {
        const metrics = getOrInitNapiMetrics();
        metrics.calls += 1;
        const rawThisArg = unwrapValue(thisArg);
        const rawArgArray = argArray.map((item) => unwrapValue(item));
        const ret = Reflect.apply(target as ESObject, rawThisArg, rawArgArray);
        return wrapValue(ret);
      },
      construct(target: ESObject, argArray: ESObject[], newTarget: ESObject) {
        const metrics = getOrInitNapiMetrics();
        metrics.calls += 1;
        const rawArgArray = argArray.map((item) => unwrapValue(item));
        const ret = Reflect.construct(target as ESObject, rawArgArray, target as ESObject);
        return wrapValue(ret);
      },
    });

    proxyCache.set(value, proxy);
    rawValueCache.set(proxy, value);
    return proxy;
  };

  (globalThis as ESObject).requireNapiPreview = (moduleName: string, isAppModule: boolean) => {
    const metrics = getOrInitNapiMetrics();
    if (!metrics.modules.includes(moduleName)) {
      metrics.modules.push(moduleName);
    }
    const loaded = originalRequireNapiPreview(moduleName, isAppModule);
    return wrapValue(loaded);
  };
}
EOF

rewrite_module_imports "@kit.ArkTS" "${SHIMS_ROOT}/arkts.test.ts"
rewrite_module_imports "@ohos.worker" "${SHIMS_ROOT}/ohos-worker.test.ts"
rewrite_module_imports "@ohos.events.emitter" "${SHIMS_ROOT}/emitter.test.ts"
rewrite_module_imports "@kit.CoreFileKit" "${SHIMS_ROOT}/corefilekit.test.ts"
rewrite_module_imports "@ohos.file.fs" "${SHIMS_ROOT}/filefs.test.js"

list_matching_files "libexample\\.so|libcompact\\.so" "${SRC_COPY_ROOT}" ts ets | while IFS= read -r file; do
  LC_ALL=C LANG=C perl -0777 -i -pe '
    s/"libexample\.so"/"example"/g;
    s/"libcompact\.so"/"compact"/g;
  ' "${file}"
done || true

list_matching_files "from\\s+\"example\"|from\\s+\"compact\"" "${SRC_COPY_ROOT}" ts ets | while IFS= read -r file; do
  LC_ALL=C LANG=C perl -0777 -i -pe '
    s#import\s*\{([^}]+)\}\s*from\s*"example";#const {$1} = requireNapiPreview("example", true);#gs;
    s#import\s*\{([^}]+)\}\s*from\s*"compact";#const {$1} = requireNapiPreview("compact", true);#gs;
    s#import\s+\*\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s+from\s*"example";#const $1 = requireNapiPreview("example", true);#g;
    s#import\s+\*\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s+from\s*"compact";#const $1 = requireNapiPreview("compact", true);#g;
  ' "${file}"
done || true

list_matching_files "requireNapiPreview" "${SRC_COPY_ROOT}" ts ets | while IFS= read -r file; do
  LC_ALL=C LANG=C perl -0777 -i -pe '
    while (s/(const\s*\{[^}]*?)\btype\s+([A-Za-z_][A-Za-z0-9_]*)(\s*,?)/$1$2$3/gs) {}
  ' "${file}"
done || true

list_matching_files "setup\.test\.js" "${SRC_COPY_ROOT}" ts ets | while IFS= read -r file; do
  LC_ALL=C LANG=C perl -0777 -i -pe 's/setup\.test\.js/setup.test/g' "${file}"
done || true

# Disable inner per-spec timeout timers in split runner mode.
# The outer harness already applies a hard timeout per suite process.
CORE_SERVICE_FILE="${SRC_COPY_ROOT}/utils/core/service.js"
if [[ -f "${CORE_SERVICE_FILE}" ]]; then
  LC_ALL=C LANG=C perl -0777 -i -pe '
    s#timer = setTimeout\(\(\) => \{\s*reject\(new Error\("execute timeout " \+ timeout \+ "ms"\)\);\s*\}, timeout\);\s*#// split runner: timeout handled by outer harness\n#g;
  ' "${CORE_SERVICE_FILE}"
fi

: > "${MANIFEST_FILE}"

find "${SRC_COPY_ROOT}" -type f \( -name "*.test.ts" -o -name "*.test.ets" \) \
  ! -path "*/utils/*" \
  ! -name "Ability.test.ets" \
  ! -name "List.test.ets" \
  ! -name "Worker.test.ts" | sort | while IFS= read -r suite_file; do
    if ! file_has_pattern "describe\\(" "${suite_file}"; then
      continue
    fi

    rel="${suite_file#${SRC_COPY_ROOT}/}"
    rel_no_ext="${rel%.*}"
    suite_id="$(echo "${rel_no_ext}" | sed -E 's#[^A-Za-z0-9]+#_#g; s#^_+##; s#_+$##')"
    entry_file="${SUITES_ROOT}/${suite_id}.ts"

    cat > "${entry_file}" <<EOF
import "../runtime/console_shim";
import suite from "../src/${rel_no_ext}";
import { runSplitSuite } from "../runtime/no_ability_runner";

runSplitSuite("${rel}", suite, "${DEFAULT_TMP_DIR}");
EOF

    echo "${suite_id}|${rel}" >> "${MANIFEST_FILE}"
  done

count="$(wc -l < "${MANIFEST_FILE}" | tr -d "[:space:]")"
echo "Source copied to: ${SRC_COPY_ROOT}"
echo "Generated split suite entries: ${count}"
echo "Suite manifest: ${MANIFEST_FILE}"
