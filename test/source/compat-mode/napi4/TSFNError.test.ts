import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";

export default () => {
  describe("CompatNapi4TSFNErrorTest", () => {
    test("should receive Rust error in threadsafe function callback", async (t) => {
      await new Promise<void>((resolve, reject) => {
        bindings.testTsfnError((err: ESObject) => {
          try {
            t.assert(err instanceof Error);
            t.is(err.message, "invalid");
            resolve();
          } catch (error) {
            reject(error);
          }
        });
      });
    });
  });
};
