import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";
import { buffer } from "@kit.ArkTS";

export default () => {
  describe("CompatSerdeSerTest", () => {
    test("serialize make_bytes_struct", (t) => {
      t.deepEqual(bindings.make_bytes_struct(), {
        code: buffer.from([0, 1, 2, 3]).buffer,
        map: "source map"
      });
    });
  });
};
