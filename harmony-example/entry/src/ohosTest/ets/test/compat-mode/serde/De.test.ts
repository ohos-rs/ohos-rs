import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";

export default () => {
  describe("CompatSerdeDeTest", () => {
    test("deserialize string", (t) => {
      t.notThrows(() => bindings.expect_hello_world("hello world"));
    });

    test("deserialize object", (t) => {
      t.notThrows(() =>
        bindings.expect_obj({
          a: 1,
          b: [1, 2],
          c: "abc",
          d: false,
          e: null,
          f: null,
          g: [9, false, "efg"],
          h: "🤷",
          i: "Empty",
          j: { Tuple: [27, "hij"] },
          k: { Struct: { a: 128, b: [9, 8, 7] } },
          l: "jkl",
          m: [0, 1, 2, 3, 4],
          o: { Value: ["z", "y", "x"] },
          p: [1, 2, 3.5],
          q: BigInt("9998881288248882845242411222333"),
          r: BigInt("-3332323888900001232323022221345")
        })
      );
    });
  });
};
