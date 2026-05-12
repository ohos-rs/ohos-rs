import { handleEntryTestWorker } from "./entry_test_worker";

const native = requireNapiPreview("worker", true);

type LaunchPayload = {
  script_url: string;
  name?: string;
  message: ESObject;
};

async function main() {
  try {
    const payload = JSON.parse(native.consumeLaunchPayload()) as LaunchPayload;
    let result: ESObject;
    switch (payload.script_url) {
      case "entry_test/ets/workers/worker.ts":
        result = await handleEntryTestWorker(payload.message);
        break;
      default:
        throw new Error(`Unsupported worker script: ${payload.script_url}`);
    }
    print(`__OHOS_WORKER_MESSAGE__${JSON.stringify(result)}`);
  } catch (err) {
    const payload = JSON.stringify({
      message: String(err?.message || err),
      stack: String(err?.stack || ""),
    });
    print(`__OHOS_WORKER_ERROR__${payload}`);
  }
}

main();
