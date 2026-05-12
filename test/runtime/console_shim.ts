import { ARK_HOST_BUNDLE_DIR } from "./ark_host_config";

if (!(globalThis as ESObject).console) {
  (globalThis as ESObject).console = {
    log: (msg: ESObject) => print(String(msg)),
    info: (msg: ESObject) => print(String(msg)),
    warn: (msg: ESObject) => print(String(msg)),
    error: (msg: ESObject) => print(String(msg)),
  };
}

function tryInstallNativeTimers() {
  if (typeof (globalThis as ESObject).setTimeout === "function") {
    return;
  }

  try {
    const loader = (globalThis as ESObject).requireNapiPreview;
    if (typeof loader !== "function") {
      return;
    }
    const etsVm = loader("ets_interop_js_napi", true);
    if (!etsVm || typeof etsVm.createRuntime !== "function") {
      return;
    }

    etsVm.createRuntime({
      "log-level": "debug",
      "panda-files": `${ARK_HOST_BUNDLE_DIR}/hello.abc`,
      "boot-panda-files": `${ARK_HOST_BUNDLE_DIR}/etsstdlib.abc:${ARK_HOST_BUNDLE_DIR}/hello.abc`,
    });
  } catch (_err) {
    // Fall back to the JS timer shim below.
  }
}

tryInstallNativeTimers();

if (!(globalThis as ESObject).setTimeout) {
  type TimerRecord = {
    dueAt: number;
    handler: ESObject;
    repeat: boolean;
    timeoutMs: number;
    args: Array<ESObject>;
  };

  let nextTimerId = 1;
  const timers = new Map<number, TimerRecord>();

  const runTimer = (id: number) => {
    Promise.resolve().then(() => {
      const timer = timers.get(id);
      if (!timer) {
        return;
      }
      if (Date.now() < timer.dueAt) {
        runTimer(id);
        return;
      }

      if (!timer.repeat) {
        timers.delete(id);
      } else {
        timer.dueAt = Date.now() + timer.timeoutMs;
      }

      if (typeof timer.handler === "function") {
        (timer.handler as (...args: Array<ESObject>) => void)(...timer.args);
      }

      if (timer.repeat && timers.has(id)) {
        runTimer(id);
      }
    });
  };

  const registerTimer = (
    handler: ESObject,
    timeoutMs: number = 0,
    repeat: boolean = false,
    ...args: Array<ESObject>
  ) => {
    const id = nextTimerId++;
    const delay = typeof timeoutMs === "number" && timeoutMs > 0 ? timeoutMs : 0;
    timers.set(id, {
      dueAt: Date.now() + delay,
      handler,
      repeat,
      timeoutMs: delay,
      args,
    });
    runTimer(id);
    return id;
  };

  (globalThis as ESObject).setTimeout = registerTimer;
  (globalThis as ESObject).setInterval = (
    handler: ESObject,
    timeoutMs: number = 0,
    ...args: Array<ESObject>
  ) => {
    return registerTimer(handler, timeoutMs, true, ...args);
  };
  (globalThis as ESObject).__ohosTimerMap__ = timers;
}

if (!(globalThis as ESObject).clearTimeout) {
  (globalThis as ESObject).clearTimeout = (id: number) => {
    const timers = (globalThis as ESObject).__ohosTimerMap__;
    if (timers && typeof timers.delete === "function") {
      timers.delete(id);
    }
  };
}

if (!(globalThis as ESObject).clearInterval) {
  (globalThis as ESObject).clearInterval = (id: number) => {
    (globalThis as ESObject).clearTimeout(id);
  };
}
