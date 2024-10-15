import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";

export default () => {
  describe("CompatNapi6BigintTest", () => {
    test("should create bigints", (t) => {
      t.is(bindings.testCreateBigintFromI64(), BigInt("9223372036854775807"));
      t.is(bindings.testCreateBigintFromU64(), BigInt("18446744073709551615"));
      t.is(
        bindings.testCreateBigintFromI128(),
        BigInt("170141183460469231731687303715884105727")
      );
      t.is(
        bindings.testCreateBigintFromU128(),
        BigInt("340282366920938463463374607431768211455")
      );
      t.is(
        bindings.testCreateBigintFromWords(),
        BigInt("-340282366920938463463374607431768211455")
      );
    });

    test("should get integers from bigints", (t) => {
      t.is(bindings.testGetBigintI64(BigInt("-123")), -123);
      t.is(bindings.testGetBigintU64(BigInt(123)), 123);
      t.deepEqual(bindings.testGetBigintWords(), [
        BigInt("9223372036854775807"),
        BigInt("9223372036854775807")
      ]);
    });
  });
};
