import { describe, test } from "../utils/setup.test";
import bindings from "../utils/compat.test";

export default () => {
  describe("CompatEitherTest", () => {
    test("either should work", (t) => {
      const fixture = "napi";
      t.is(bindings.eitherNumberString(1), 101);
      t.is(bindings.eitherNumberString(fixture), `Either::B(${fixture})`);
    });

    test("dynamic argument length should work", (t) => {
      t.is(bindings.dynamicArgumentLength(1), 101);
      t.is(bindings.dynamicArgumentLength(), 42);
    });
  });
};
