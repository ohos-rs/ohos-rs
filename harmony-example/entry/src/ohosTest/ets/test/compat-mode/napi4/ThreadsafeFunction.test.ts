import { describe, test } from "../../utils/setup.test";
import bindings from "../../utils/compat.test";

async function main() {
  await Promise.resolve();
  const a1 = new bindings.A((err, s) => {
    console.info(s);
  });
  const a2 = new bindings.A((err, s) => {
    console.info(s);
  });
  a1.call();
  a2.call();
}

export default () => {
  describe("CompatNapi4ThreadsafeFunctionTest", () => {
    test("should get js function called from a thread", async (t) => {
      let called = 0;

      await new Promise<void>((resolve, reject) => {
        bindings.testThreadsafeFunction((...args: any[]) => {
          called += 1;
          try {
            if (args[1][0] === 0) {
              t.deepEqual(args, [null, [0, 1, 2, 3]]);
            } else {
              t.deepEqual(args, [null, [3, 2, 1, 0]]);
            }
          } catch (err) {
            reject(err);
          }

          if (called === 2) {
            resolve();
          }
        });
      });
    });

    test("should be able to throw error in tsfn", (t) => {
      t.notThrows(() =>
        bindings.testThreadsafeFunction(() => {
          throw Error("Throw in thread safe function");
        })
      );
    });

    test("tsfn dua instance", (t) => {
      t.notThrowsAsync(async () => {
        main().catch((e) => {
          console.error(e);
        });
      });
    });
  });
};
