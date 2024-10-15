import { describe, test } from "../utils/setup.test";
import bindings from "../utils/compat.test";
import { spy } from "../utils/mock.test";

function wait(delay: number) {
  return new Promise((resolve) => setTimeout(resolve, delay));
}

const delay = 100;

export default () => {
  describe("CompatGlobalTest", () => {
    test("should setTimeout", async (t) => {
      const cbSpy = spy();
      bindings.setTimeout(cbSpy.func, delay);
      cbSpy.mocker.verify("test", []).times(0);
      await wait(delay + 10);
      cbSpy.mocker.verify("test", []).times(1);
    });

    test("should clearTimeout", async (t) => {
      const cbSpy = spy();
      const timer = setTimeout(() => cbSpy.func(), delay);
      cbSpy.mocker.verify("test", []).times(0);
      bindings.clearTimeout(timer);
      await wait(delay + 10);
      cbSpy.mocker.verify("test", []).times(0);
    });
  });
};
