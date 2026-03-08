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
