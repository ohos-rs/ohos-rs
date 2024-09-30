import { expect, it as test, describe } from "./framework.test";
import type { TestFn } from "./types/test-fn";

const testContext: ESObject = {
  is: (actual: ESObject, expected: ESObject) => {
    expect(actual).assertEqual(expected);
  },
  deepEqual: (actual: ESObject, expected: ESObject) => {
    expect(actual).assertDeepEquals(expected);
  },
  throws: (fn: ESObject, expected: ESObject) => {
    expect(fn).assertThrowError(expected);
  },
  notThrows: (fn: ESObject, expected: ESObject) => {
    expect(fn).not().assertThrowError(expected);
  },
  throwsAsync: async (fn: ESObject, expected: ESObject) => {
    let ret = fn instanceof Promise ? fn : await fn();
    if (expected) {
      expect(ret).assertPromiseIsRejectedWith(expected);
    } else {
      expect(ret).assertPromiseIsRejected();
    }
  },
  notThrowsAsync: async (fn: ESObject, expected: ESObject) => {
    let ret = fn instanceof Promise ? fn : await fn();
    if (expected) {
      expect(ret).assertPromiseIsResolvedWith(expected);
    } else {
      expect(ret).assertPromiseIsResolved();
    }
  },
  true: (actual: ESObject, message: ESObject) => {
    expect(actual).assertEqual(true);
  },
  false: (actual: ESObject, message: ESObject) => {
    expect(actual).assertEqual(false);
  }
};

const testRunner = ((title: ESObject, spec: ESObject) => {
  test(title, 0, async (done) => {
    spec(testContext);
    done();
  });
}) as TestFn;

export { testRunner as test, describe };
