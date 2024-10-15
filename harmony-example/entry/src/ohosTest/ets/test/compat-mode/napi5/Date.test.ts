import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";

export default () => {
  describe("CompatNapi5DateTest", () => {
    test("should return false if value is not date", (t) => {
      t.false(bindings.testObjectIsDate({}));
      t.false(bindings.testObjectIsDate(null));
      t.false(bindings.testObjectIsDate());
      t.false(bindings.testObjectIsDate(10249892));
    });

    test("should return true if value is date", (t) => {
      t.true(bindings.testObjectIsDate(new Date()));
    });

    test("should create date", (t) => {
      const timestamp = new Date().valueOf();
      t.deepEqual(bindings.testCreateDate(timestamp), new Date(timestamp));
    });

    test("should get date value", (t) => {
      const date = new Date();
      t.is(bindings.testGetDateValue(date), date.valueOf());
    });
  });
};
