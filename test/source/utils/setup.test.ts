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

function isArrayBufferLike(value: ESObject): boolean {
  return value instanceof ArrayBuffer || ArrayBuffer.isView(value);
}

function toUint8Bytes(value: ESObject): Uint8Array {
  if (value instanceof Uint8Array) {
    return value;
  }
  if (ArrayBuffer.isView(value)) {
    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
  }
  return new Uint8Array(value as ArrayBuffer);
}

function deepEqualValue(actual: ESObject, expected: ESObject): boolean {
  if (Object.is(actual, expected)) {
    return true;
  }

  if (isArrayBufferLike(actual) && isArrayBufferLike(expected)) {
    const left = toUint8Bytes(actual);
    const right = toUint8Bytes(expected);
    if (left.length !== right.length) {
      return false;
    }
    for (let i = 0; i < left.length; i++) {
      if (left[i] !== right[i]) {
        return false;
      }
    }
    return true;
  }

  if (Array.isArray(actual) && Array.isArray(expected)) {
    if (actual.length !== expected.length) {
      return false;
    }
    for (let i = 0; i < actual.length; i++) {
      if (!deepEqualValue(actual[i], expected[i])) {
        return false;
      }
    }
    return true;
  }

  if (
    actual !== null &&
    expected !== null &&
    typeof actual === "object" &&
    typeof expected === "object"
  ) {
    const actualKeys = Object.keys(actual as ESObject);
    const expectedKeys = Object.keys(expected as ESObject);
    if (actualKeys.length !== expectedKeys.length) {
      return false;
    }
    for (const key of actualKeys) {
      if (!Object.prototype.hasOwnProperty.call(expected, key)) {
        return false;
      }
      if (!deepEqualValue((actual as ESObject)[key], (expected as ESObject)[key])) {
        return false;
      }
    }
    return true;
  }

  return false;
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
    expect(Object.is(actual, expected)).assertEqual(true);
  },
  deepEqual: (actual: ESObject, expected: ESObject) => {
    markAssertionCall();
    expect(deepEqualValue(actual, expected)).assertEqual(true);
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
    let capturedError: ESObject = undefined;
    const ret = toPromise(fn).catch((err) => {
      capturedError = err;
      throw err;
    });
    if (expected) {
      await expect(ret).assertPromiseIsRejectedWith(expected);
    } else {
      await expect(ret).assertPromiseIsRejected();
    }
    return capturedError;
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
    expect(Object.is(actual, expected)).assertEqual(false);
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
