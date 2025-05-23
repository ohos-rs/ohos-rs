import { describe, test } from "../utils/setup.test";
import bindings from "../utils/compat.test";

export default () => {
  describe("CompatFunctionTest", () => {
    test("should call the function", (t) => {
      bindings.testCallFunction((arg1: string, arg2: string) => {
        t.is(`${arg1} ${arg2}`, "hello world");
      });
    });

    test("should call function with ref args", (t) => {
      bindings.testCallFunctionWithRefArguments(
        (arg1: string, arg2: string) => {
          t.is(`${arg1} ${arg2}`, "hello world");
        }
      );
    });

    test('should set "this" properly', (t) => {
      const obj = {};
      bindings.testCallFunctionWithThis.call(obj, function (this: typeof obj) {
        t.is(this, obj);
      });
    });

    test("should handle errors", (t) => {
      bindings.testCallFunctionError(
        () => {
          throw new Error("Testing");
        },
        (err: Error) => {
          t.is(err.message, "Testing");
        }
      );
    });

    test("should be able to create function from closure", (t) => {
      for (let i = 0; i < 100; i++) {
        t.is(
          bindings.testCreateFunctionFromClosure()(
            ...Array.from({ length: i }, (_, i) => i)
          ),
          `arguments length: ${i}`
        );
      }
    });
  });
};
