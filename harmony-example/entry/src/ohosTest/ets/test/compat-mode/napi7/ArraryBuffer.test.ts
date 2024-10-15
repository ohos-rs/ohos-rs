import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";
import { buffer } from "@kit.ArkTS";

export default () => {
  describe("CompatNapi7ArrayBufferTest", () => {
    test("should be able to detach ArrayBuffer", (t) => {
      const buf = buffer.from("hello world");
      const ab = buf.buffer.slice(0, buf.length);
      try {
        bindings.testDetachArrayBuffer(ab);
        t.is(ab.byteLength, 0);
      } catch (e) {
        console.log(`hello_test: ${e.code} ${e.message}`);
      }
    });

    test("is detached arraybuffer should work fine", (t) => {
      const buf = buffer.from("hello world");
      const ab = buf.buffer.slice(0, buf.length);
      bindings.testDetachArrayBuffer(ab);
      const nonDetachedArrayBuffer = new ArrayBuffer(10);
      t.true(bindings.testIsDetachedArrayBuffer(ab));
      t.false(bindings.testIsDetachedArrayBuffer(nonDetachedArrayBuffer));
    });
  });
};
