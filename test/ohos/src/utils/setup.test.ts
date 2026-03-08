import { expect, it as test, describe } from "./framework.test";
import type { TestFn } from "./types/test-fn";

type SplitMetrics = {
  registered: number;
  executed: number;
  assertionCalls: number;
  noAssertionTests: number;
  currentTest: string;
};

function getSplitMetrics(): SplitMetrics {
  const g = globalThis as ESObject;
  if (!g.__ohosSplitMetrics__) {
    g.__ohosSplitMetrics__ = {
      registered: 0,
      executed: 0,
      assertionCalls: 0,
      noAssertionTests: 0,
      currentTest: "",
    };
  }
  return g.__ohosSplitMetrics__ as SplitMetrics;
}

function markAssertionCall() {
  const metrics = getSplitMetrics();
  metrics.assertionCalls += 1;
}

function toPromise(fnOrPromise: ESObject): Promise<ESObject> {
  if (fnOrPromise instanceof Promise) {
    return fnOrPromise;
  }

  if (typeof fnOrPromise === "function") {
    try {
      return Promise.resolve(fnOrPromise());
    } catch (error) {
      return Promise.reject(error);
    }
  }

  return Promise.resolve(fnOrPromise);
}

const testContext: ESObject = {
  is: (actual: ESObject, expected: ESObject) => {
    markAssertionCall();
    expect(actual).assertEqual(expected);
  },
  deepEqual: (actual: ESObject, expected: ESObject) => {
    markAssertionCall();
    expect(actual).assertDeepEquals(expected);
  },
  assert: (actual: ESObject, _message?: ESObject) => {
    markAssertionCall();
    expect(Boolean(actual)).assertEqual(true);
  },
  throws: (fn: ESObject, expected: ESObject) => {
    markAssertionCall();
    expect(fn).assertThrowError(expected);
  },
  notThrows: (fn: ESObject, expected: ESObject) => {
    markAssertionCall();
    expect(fn).not().assertThrowError(expected);
  },
  throwsAsync: async (fn: ESObject, expected: ESObject) => {
    markAssertionCall();
    const ret = toPromise(fn);
    if (expected) {
      await expect(ret).assertPromiseIsRejectedWith(expected);
    } else {
      await expect(ret).assertPromiseIsRejected();
    }
  },
  notThrowsAsync: async (fn: ESObject, expected: ESObject) => {
    markAssertionCall();
    const ret = toPromise(fn);
    if (expected) {
      await expect(ret).assertPromiseIsResolvedWith(expected);
    } else {
      await expect(ret).assertPromiseIsResolved();
    }
  },
  true: (actual: ESObject, _message: ESObject) => {
    markAssertionCall();
    expect(actual).assertEqual(true);
  },
  false: (actual: ESObject, _message: ESObject) => {
    markAssertionCall();
    expect(actual).assertEqual(false);
  },
  not: (actual: ESObject, expected: ESObject) => {
    markAssertionCall();
    expect(actual).not().assertDeepEquals(expected);
  },
  regex: (actual: ESObject, expected: ESObject) => {
    markAssertionCall();
    if (!(expected instanceof RegExp)) {
      throw new Error("t.regex expects a RegExp");
    }
    expect(expected.test(String(actual))).assertEqual(true);
  },
  fail: (message?: ESObject) => {
    markAssertionCall();
    throw new Error(message ? String(message) : "Test failed");
  },
};

const testRunner = ((title: ESObject, spec: ESObject) => {
  const metrics = getSplitMetrics();
  metrics.registered += 1;
  test(title, 0, async (done) => {
    metrics.currentTest = String(title);
    metrics.executed += 1;
    if (typeof print === "function") {
      print(`[CASE] start ${metrics.currentTest}`);
    }

    try {
      await spec(testContext);
    } finally {
      if (typeof print === "function") {
        print(`[CASE] end ${String(title)}`);
      }
      metrics.currentTest = "";
    }
    done();
  });
}) as TestFn;

export { testRunner as test, describe };
