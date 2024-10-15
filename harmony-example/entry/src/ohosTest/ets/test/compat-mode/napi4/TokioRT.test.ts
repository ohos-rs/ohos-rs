import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";
import { buffer } from "@kit.ArkTS";
import { EXAMPLE_STRING, EXAMPLE_TXT_FILE_NAME } from "../../utils/file.test";

export default (path) => {
  const txtPath = path + EXAMPLE_TXT_FILE_NAME;
  describe("CompatNapi4TokioRTTest", () => {
    test("should execute future on tokio runtime", async (t) => {
      const fileContent = await bindings.testExecuteTokioReadfile(txtPath);
      t.true(buffer.isBuffer(fileContent));
      t.deepEqual(buffer.from(EXAMPLE_STRING).buffer, fileContent);
    });

    test("should reject error from tokio future", async (t) => {
      try {
        await bindings.testTokioError(txtPath);
        throw new TypeError("Unreachable");
      } catch (e) {
        t.is((e as Error).message, "Error from tokio future");
      }
    });

    test("should be able to execute future paralleled", async (t) => {
      const buffers = await Promise.all(
        Array.from({ length: 50 }).map((_) =>
          bindings.testExecuteTokioReadfile(txtPath)
        )
      );
      for (const fileContent of buffers) {
        t.true(buffer.isBuffer(fileContent));
        t.deepEqual(buffer.from(EXAMPLE_STRING).buffer, fileContent);
      }
    });
  });
};
