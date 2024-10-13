import { describe, test } from "../utils/setup.test";

import {
  validateArray,
  validateTypedArray,
  validateTypedArraySlice,
  validateBufferSlice,
  validateBigint,
  validateBuffer,
  validateBoolean,
  validateDate,
  validateDateTime,
  createExternal,
  validateExternal,
  validateFunction,
  validateHashMap,
  validatePromise,
  validateString,
  validateNull,
  validateUndefined,
  validateEnum,
  validateStringEnum,
  KindInValidate,
  StatusInValidate,
  returnUndefinedIfInvalid,
  returnUndefinedIfInvalidPromise,
  validateOptional
} from "libexample.so";
import { buffer } from "@kit.ArkTS";

export default () => {
  describe("StrictsTest", () => {
    test("should validate array", (t) => {
      t.is(validateArray([1, 2, 3]), 3);

      t.throws(() => validateArray(1 as ESObject), {
        message: "Expected an array",
        code: "InvalidArg"
      });
    });

    test("should validate arraybuffer", (t) => {
      t.is(validateTypedArray(new Uint8Array([1, 2, 3])), 3);

      t.throws(() => validateTypedArray(1 as ESObject), {
        code: "InvalidArg",
        message: "Expected a TypedArray value"
      });

      t.is(validateTypedArraySlice(new Uint8Array([1, 2, 3])), 3);

      t.throws(() => validateTypedArraySlice(1 as ESObject), {
        code: "InvalidArg",
        message: "Expected a TypedArray value"
      });

      t.is(validateBufferSlice(buffer.from("hello").buffer), 5);

      t.throws(() => validateBufferSlice(2 as ESObject), {
        code: "InvalidArg",
        message: "Expected a Buffer value"
      });
    });

    test("should validate BigInt", (t) => {
      const fx = BigInt(1024 * 1024 * 1024 * 1024);
      t.is(validateBigint(fx), fx);

      t.throws(() => validateBigint(1 as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be BigInt, but received Number"
      });
    });

    test("should validate buffer", (t) => {
      t.is(validateBuffer(buffer.from("hello").buffer), 5);

      t.throws(() => validateBuffer(2 as ESObject), {
        code: "InvalidArg",
        message: "Expected a Buffer value"
      });
    });

    test("should validate boolean value", (t) => {
      t.is(validateBoolean(true), false);
      t.is(validateBoolean(false), true);

      t.throws(() => validateBoolean(1 as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be Boolean, but received Number"
      });
    });

    test("should validate date", (t) => {
      const fx = new Date("2016-12-24");
      t.is(validateDate(fx), fx.valueOf());
      t.is(validateDateTime(fx), 1);

      t.throws(() => validateDate(1 as ESObject), {
        code: "InvalidArg",
        message: "Expected a Date object"
      });

      t.throws(() => validateDateTime(2 as ESObject), {
        code: "InvalidArg",
        message: "Expected a Date object"
      });
    });

    test("should validate External", (t) => {
      const fx = createExternal(1);
      t.is(validateExternal(fx), 1);

      t.throws(() => validateExternal(1 as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be External, but received Number"
      });
    });

    test("should validate function", (t) => {
      t.is(
        validateFunction(() => 1),
        4
      );

      t.throws(() => validateFunction(2 as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be Function, but received Number"
      });
    });

    test("should validate Map", (t) => {
      t.is(
        validateHashMap({
          a: 1,
          b: 2
        }),
        2
      );

      t.throws(() => validateHashMap(undefined as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be Object, but received Undefined"
      });
    });

    test("should validate promise", async (t) => {
      t.is(
        await validatePromise(
          new Promise((resolve) => {
            setTimeout(() => {
              resolve(1);
            }, 100);
          })
        ),
        2
      );

      await t.throwsAsync(() => validatePromise(1 as ESObject), {
        code: "InvalidArg",
        message: "Expected Promise object"
      });
    });

    test("should validate string", (t) => {
      t.is(validateString("hello"), "hello!");

      t.throws(() => validateString(1 as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be String, but received Number"
      });
    });

    test("should validate null", (t) => {
      t.notThrows(() => validateNull(null));

      t.throws(() => validateNull(1 as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be Null, but received Number"
      });
    });

    test("should validate undefined", (t) => {
      t.notThrows(() => validateUndefined(void 0));

      t.notThrows(() => validateUndefined(undefined as ESObject));

      t.throws(() => validateUndefined(1 as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be Undefined, but received Number"
      });
    });

    test("should validate enum", (t) => {
      t.is(validateEnum(KindInValidate.Cat), KindInValidate.Cat);

      t.throws(() => validateEnum("3" as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be Number, but received String"
      });

      t.is(validateStringEnum(StatusInValidate.Poll), "Poll");

      t.throws(() => validateStringEnum(1 as ESObject), {
        code: "InvalidArg",
        message: "Expect value to be String, but received Number"
      });
    });

    test("should return undefined if arg is invalid", (t) => {
      t.is(returnUndefinedIfInvalid(true), false);

      t.is(returnUndefinedIfInvalid(1 as ESObject), undefined);
    });

    test("should return Promise.reject() if arg is not Promise", async (t) => {
      t.is(await returnUndefinedIfInvalidPromise(Promise.resolve(true)), false);

      await t.throwsAsync(() => returnUndefinedIfInvalidPromise(1 as ESObject));
    });

    test("should validate Option<T>", (t) => {
      t.is(validateOptional(null, null), false);
      t.is(validateOptional(null, false), false);
      t.is(validateOptional("1", false), true);
      t.is(validateOptional(null, true), true);

      t.throws(() => validateOptional(1 as ESObject, null));

      t.throws(() => validateOptional(null, 2 as ESObject));

      t.throws(() => validateOptional(1 as ESObject, 2 as ESObject));
    });
  });
};
