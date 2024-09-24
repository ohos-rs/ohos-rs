import { expect, it as test, describe } from '@ohos/hypium';
import type { TestFn } from './types/test-fn';

const testContext: ESObject = {
  is: (actual: ESObject, expected: ESObject) => {
    expect(actual).assertEqual(expected)
  },
  deepEqual: (actual: ESObject, expected: ESObject) => {
    expect(actual).assertEqual(expected)
  },
  throws: (fn: ESObject, expected: ESObject) => {
    if (expected) {
      expect(fn).assertThrowError(expected)
    } else {
      expect(fn).not().assertThrowError(expected)
    }
  },
  notThrows: (fn: ESObject, expected: ESObject) => {
    if (expected) {
      expect(fn).assertThrowError(expected)
    } else {
      expect(fn).not().assertThrowError(expected)
    }
  },
  throwsAsync: async (fn: ESObject, expected: ESObject) => {
    if (expected) {
      expect(fn instanceof Promise ? fn : await fn()).assertPromiseIsRejectedWith(expected)
    } else {
      expect(fn instanceof Promise ? fn : await fn()).assertPromiseIsRejected()
    }
  },
  notThrowsAsync: async (fn: ESObject, expected: ESObject) => {
    if (expected) {
      expect(fn instanceof Promise ? fn : await fn()).assertPromiseIsResolvedWith(expected)
    } else {
      expect(fn instanceof Promise ? fn : await fn()).assertPromiseIsResolved()
    }
  },
  true: (actual: ESObject, message: ESObject) => {
    expect(actual).assertEqual(true)
  },
  false: (actual: ESObject, message: ESObject) => {
    expect(actual).assertEqual(false)
  },
}

const testRunner = ((title: ESObject, spec: ESObject) => {
  test(title, 0, async () => {
    await Promise.resolve(spec(testContext))
  })
}) as TestFn;


export { testRunner as test, describe }