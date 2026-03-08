const { Animal, Kind, DEFAULT_COST, bufferPassThrough, arrayBufferPassThrough } = requireNapiPreview(
  "example",
  true,
);
import { describe, test } from "../utils/setup.test";
import { worker, buffer } from "../utils/shims/arkts.test";

const isSplitMode = Boolean((globalThis as ESObject).__ohosSplitImmediateExecute__);

function runSplitModeSuite() {
  describe("WorkerTest", () => {
    test("should be able to require in worker thread", (t) => {
      const expected = Animal.withKind(Kind.Cat).whoami() + DEFAULT_COST;
      for (let i = 0; i < 20; i++) {
        t.is(Animal.withKind(Kind.Cat).whoami() + DEFAULT_COST, expected);
      }
    });

    test("custom GC works on worker_threads", async (t) => {
      for (let i = 0; i < 20; i++) {
        const [bufferResult, arrayBufferResult] = await Promise.all([
          Promise.all(
            Array.from({
              length: 100,
            }).map(() => bufferPassThrough(buffer.from([1, 2, 3]).buffer)),
          ).then(() => "done"),
          Promise.all(
            Array.from({
              length: 100,
            }).map(() => arrayBufferPassThrough(Uint8Array.from([1, 2, 3]))),
          ).then(() => "done"),
        ]);
        t.is(bufferResult, "done");
        t.is(arrayBufferResult, "done");
      }
    });

    test("should be able to new Class in worker thread concurrently", (t) => {
      for (let i = 0; i < 20; i++) {
        const ellie = new Animal(Kind.Cat, "Ellie");
        t.is(ellie.name, "Ellie");
      }
    });
  });
}

function runOhosWorkerSuite() {
  describe("WorkerTest", () => {
    test("should be able to require in worker thread", async (t) => {
      await Promise.all(
        Array.from({
          length: 20,
        }).map(() => {
          const w = new worker.ThreadWorker("entry_test/ets/workers/worker.ts");
          return new Promise<void>((resolve, reject) => {
            w.postMessage({
              type: "require",
            });
            w.onmessage = (msg) => {
              t.is(msg.data, Animal.withKind(Kind.Cat).whoami() + DEFAULT_COST);
              resolve();
            };
            w.onerror = (err) => {
              reject(err);
            };
          }).then(() => w.terminate());
        }),
      );
    });

    test("custom GC works on worker_threads", async (t) => {
      await Promise.all(
        Array.from({
          length: 20,
        }).map(() =>
          Promise.all([
            new Promise<worker.ThreadWorker>((resolve, reject) => {
              const w = new worker.ThreadWorker("entry_test/ets/workers/worker.ts");
              w.postMessage({
                type: "async:buffer",
              });
              w.onmessage = (msg) => {
                t.is(msg.data, "done");
                resolve(w);
              };
              w.onerror = (err) => {
                reject(err);
              };
            }).then((w) => {
              return w.terminate();
            }),
            new Promise<worker.ThreadWorker>((resolve, reject) => {
              const w = new worker.ThreadWorker("entry_test/ets/workers/worker.ts");
              w.postMessage({
                type: "async:arraybuffer",
              });
              w.onmessage = (msg) => {
                t.is(msg.data, "done");
                resolve(w);
              };
              w.onerror = (err) => {
                reject(err);
              };
            }).then((w) => {
              return w.terminate();
            }),
          ]),
        ),
      );
    });

    test("should be able to new Class in worker thread concurrently", async (t) => {
      await Promise.all(
        Array.from({
          length: 20,
        }).map(() => {
          const w = new worker.ThreadWorker("entry_test/ets/workers/worker.ts");
          return new Promise<void>((resolve, reject) => {
            w.postMessage({
              type: "constructor",
            });
            w.onmessage = (msg) => {
              t.is(msg.data, "Ellie");
              resolve();
            };
            w.onerror = (err) => {
              reject(err);
            };
          }).then(() => w.terminate());
        }),
      );
    });
  });
}

export default () => {
  if (isSplitMode) {
    runSplitModeSuite();
    return;
  }
  runOhosWorkerSuite();
};
