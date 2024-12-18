import { describe, test } from "../utils/setup.test";
import bindings from "../utils/compat.test";
import { buffer } from "@kit.ArkTS";

const FIXTURE_STRING = " + Rust 🦀 string!";

export default () => {
  describe("CompatStringTest", () => {
    test("should be able to concat string", (t) => {
      const fixture = "JavaScript 🌳 你好 napi";
      t.is(bindings.concatString(fixture), fixture + FIXTURE_STRING);
    });

    test("should be able to concat string with char \0", (t) => {
      const fixture = "JavaScript \0 🌳 你好 \0 napi";
      t.is(bindings.concatString(fixture), fixture + FIXTURE_STRING);
    });

    test("should be able to concat utf16 string", (t) => {
      const fixture = "JavaScript 🌳 你好 napi";
      t.is(bindings.concatUTF16String(fixture), fixture + FIXTURE_STRING);
    });

    test("should be able to concat latin1 string", (t) => {
      const fixture = "æ¶½¾DEL";
      t.is(bindings.concatLatin1String(fixture), "æ¶½¾DEL    " + FIXTURE_STRING);
    });

    test("should be able to crate latin1 string", (t) => {
      const ret = bindings.createLatin1();
      t.is(ret, buffer.from([169, 191]).toString("latin1"));
    });
  });
};
