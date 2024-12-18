import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";

export default () => {
  describe("CompatNapi8ObjectTest", () => {
    test("should be able to freeze object", (t) => {
      const obj: any = {};
      bindings.testFreezeObject(obj);
      t.true(Object.isFrozen(obj));
      t.throws(() => {
        obj.a = 1;
      });
    });

    test("should be able to seal object", (t) => {
      const obj: any = {};
      bindings.testSealObject(obj);
      t.true(Object.isSealed(obj));
      t.throws(() => {
        obj.a = 1;
      });
    });
  });
};
