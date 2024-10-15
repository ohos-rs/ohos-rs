import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";

export default () => {
  describe("CompatNapi6InstanceDataTest", () => {
    test("should set and get instance data", (t) => {
      t.is(bindings.getInstanceData(), undefined);
      bindings.setInstanceData();
      t.is(bindings.getInstanceData(), 1024);
    });

    test("should throw if get instance data type mismatched", (t) => {
      t.throws(bindings.getWrongTypeInstanceData);
    });
  });
};
