import { describe, test } from "../utils/setup.test";

import { NotWritableClass } from "libexample.so";

export default function objectAttrTest() {
  describe("ObjectAttrTest", () => {
    test("Not Writable Class", (t) => {
      const obj = new NotWritableClass("1");
      t.throws(() => {
        obj.name = "2";
      });
      obj.setName("2");
      t.is(obj.name, "2");
      t.throws(() => {
        obj.setName = () => {};
      });
    });
  });
}
