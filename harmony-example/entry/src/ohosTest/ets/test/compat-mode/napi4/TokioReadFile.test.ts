import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";
import { buffer } from "@kit.ArkTS";
import { EXAMPLE_TXT_FILE_NAME, EXAMPLE_STRING } from "../../utils/file.test";

export default (path) => {
  describe("CompatNapi4TokioReadFileTest", () => {
    test("should read a file and return its a buffer", async (t) => {
      await new Promise<void>((resolve, reject) => {
        bindings.testTokioReadfile(
          path + EXAMPLE_TXT_FILE_NAME,
          (err: Error | null, value) => {
            try {
              t.is(err, null);
              t.is(buffer.isBuffer(value), true);
              t.is(buffer.from(value).toString(), EXAMPLE_STRING);
              resolve();
            } catch (err) {
              reject(err);
            }
          }
        );
      });
    });
  });
};
