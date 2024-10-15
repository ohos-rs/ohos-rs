import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";

export default () => {
  describe("CompatNapi4DeferredTest", () => {
    test("should resolve deferred from background thread", async (t) => {
      const promise = bindings.testDeferred(false);
      t.assert(promise instanceof Promise);

      const result = await promise;
      t.is(result, 15);
    });

    test("should reject deferred from background thread", async (t) => {
      await t.throwsAsync(() => bindings.testDeferred(true), {
        message: "Fail"
      });
    });
  });
};
