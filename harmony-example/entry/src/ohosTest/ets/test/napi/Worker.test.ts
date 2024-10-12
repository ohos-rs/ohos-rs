import { Animal, Kind, DEFAULT_COST } from "libexample.so";
import { describe, test } from "../utils/setup.test.js";
import { worker } from "@kit.ArkTS";

export default () => {
  describe("WorkerTest", () => {
    test("should be able to require in worker thread", async (t) => {
      await Promise.all(
        Array.from({
          length: 20
        }).map(() => {
          const w = new worker.ThreadWorker("entry_test/ets/workers/worker.ts");
          return new Promise<void>((resolve, reject) => {
            w.postMessage({
              type: "require"
            });
            w.onmessage = (msg) => {
              t.is(msg.data, Animal.withKind(Kind.Cat).whoami() + DEFAULT_COST);
              resolve();
            };
            w.onerror = (err) => {
              reject(err);
            };
          }).then(() => w.terminate());
        })
      );
    });

    test("custom GC works on worker_threads", async (t) => {
      await Promise.all(
        Array.from({
          length: 20
        }).map(() =>
          Promise.all([
            new Promise<worker.ThreadWorker>((resolve, reject) => {
              const w = new worker.ThreadWorker(
                "entry_test/ets/workers/worker.ts"
              );
              w.postMessage({
                type: "async:buffer"
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
              const w = new worker.ThreadWorker(
                "entry_test/ets/workers/worker.ts"
              );
              w.postMessage({
                type: "async:arraybuffer"
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
            })
          ])
        )
      );
    });

    test("should be able to new Class in worker thread concurrently", async (t) => {
      await Promise.all(
        Array.from({
          length: 20
        }).map(() => {
          const w = new worker.ThreadWorker("entry_test/ets/workers/worker.ts");
          return new Promise<void>((resolve, reject) => {
            w.postMessage({
              type: "constructor"
            });
            w.onmessage = (msg) => {
              t.is(msg.data, "Ellie");
              resolve();
            };
            w.onerror = (err) => {
              reject(err);
            };
          }).then(() => w.terminate());
        })
      );
    });
  });
};
