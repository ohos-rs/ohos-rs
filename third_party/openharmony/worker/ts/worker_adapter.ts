import { handleEntryTestWorker } from "../runtime/entry_test_worker";

const native = requireNapiPreview("worker", true);

type WorkerMessage = {
  data: ESObject;
};

type WorkerError = {
  message: string;
};

type WorkerHandler = (message: ESObject) => Promise<ESObject>;

function toWorkerError(err: ESObject): WorkerError {
  return {
    message: String(err?.message || err),
  };
}

function resolveHandler(scriptURL: string): WorkerHandler {
  switch (scriptURL) {
    case "entry_test/ets/workers/worker.ts":
      return handleEntryTestWorker;
    default:
      return async () => {
        throw new Error(`Unsupported worker script: ${scriptURL}`);
      };
  }
}

class ThreadWorker {
  private readonly id: number;
  private readonly scriptURL: string;
  private active = true;
  name: string;
  onmessage?: (event: WorkerMessage) => void;
  onerror?: (error: WorkerError) => void;
  onmessageerror?: (error: WorkerError) => void;

  constructor(scriptURL: string, options?: ESObject) {
    this.scriptURL = scriptURL;
    this.name = options?.name || scriptURL || "worker";
    this.id = native.createThreadWorker(scriptURL, this.name);
  }

  postMessage(data: ESObject) {
    const handler = resolveHandler(this.scriptURL);
    native
      .postMessage(this.id, data)
      .then(() => handler(data))
      .then((result: ESObject) => {
        if (!this.active) {
          return;
        }
        this.onmessage?.({ data: result });
      })
      .catch((err: ESObject) => {
        if (!this.active) {
          return;
        }
        const payload = toWorkerError(err);
        this.onerror?.(payload);
        this.onmessageerror?.(payload);
      });
  }

  terminate() {
    this.active = false;
    native.terminateThreadWorker(this.id);
    return Promise.resolve();
  }
}

export default {
  ThreadWorker,
};
