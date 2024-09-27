import { describe, test } from "../utils/setup.test";
import { receiveString } from "libexample.so";

export default function errorMessage() {
  describe("ActsErrorMessageTest", () => {
    test("message", (t) => {
      t.throws(() => receiveString(function a() {} as ESObject), {
        message:
          "Failed to convert JavaScript value `function a(..) ` into rust type `String`"
      });

      t.throws(() => receiveString((() => {}) as ESObject), {
        message:
          "Failed to convert JavaScript value `function anonymous(..) ` into rust type `String`"
      });

      t.throws(() => receiveString(1 as ESObject), {
        message:
          "Failed to convert JavaScript value `Number 1 ` into rust type `String`"
      });
      t.throws(
        () =>
          receiveString({
            a: 1,
            b: {
              foo: "bar",
              s: false
            }
          } as ESObject),
        {
          message:
            'Failed to convert JavaScript value `Object {"a":1,"b":{"foo":"bar","s":false}}` into rust type `String`'
        }
      );
      // t.throws(() => receiveString(Symbol('1') as ESObject), {
      //   message:
      //   'Failed to convert JavaScript value `Symbol` into rust type `String`',
      // })

      t.throws(() => receiveString(undefined as ESObject), {
        message:
          "Failed to convert JavaScript value `Undefined` into rust type `String`"
      });

      t.throws(() => receiveString(null), {
        message:
          "Failed to convert JavaScript value `Null` into rust type `String`"
      });

      t.throws(() => receiveString(100n as ESObject), {
        message:
          "Failed to convert JavaScript value `BigInt 100 ` into rust type `String`"
      });
    });
  });
}
