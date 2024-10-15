import { describe, test } from "../utils/setup.test";
import bindings from "../utils/compat.test";
import { buffer } from "@kit.ArkTS";

export default () => {
  describe("CompatSpawnTest", () => {
    test("should be able to spawn thread and return promise", async (t) => {
      const result = await bindings.testSpawnThread(20);
      t.is(result, 6765);
    });

    test("should be able to spawn thread with ref value", async (t) => {
      const fixture = "hello";
      const result = await bindings.testSpawnThreadWithRef(
        buffer.from(fixture).buffer
      );
      t.is(result, fixture.length);
    });

    test("should be able to spawn with error", async (t) => {
      const fixture = Array.from({ length: 10 }).fill("0").join("");
      const err = new Error("Unreachable");
      try {
        await bindings.testSpawnThreadWithRef(buffer.from(fixture).buffer);
        throw err;
      } catch (e) {
        t.not(e, err);
      }
    });
  });
};
