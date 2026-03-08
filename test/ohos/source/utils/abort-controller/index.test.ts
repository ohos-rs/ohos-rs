import emitter from "@ohos.events.emitter";

export class AbortSignal {
  onabort: null | ((this: AbortSignal, event: ESObject) => void);
  aborted: boolean;
  reason: ESObject;

  constructor() {
    this.onabort = null;
    this.aborted = false;
    this.reason = undefined;
  }
  toString = () => {
    return "[object AbortSignal]";
  };
  removeEventListener = (name: string, handler: ESObject) => {
    emitter.off(name, handler);
  };
  addEventListener = (name: string, handler: ESObject) => {
    emitter.on(name, handler);
  };
  dispatchEvent = (type: string) => {
    const event: ESObject = { type, target: this };
    const handlerName = `on${type}`;

    if (typeof (this as ESObject)[handlerName] === "function")
      (this as ESObject)[handlerName](event);

    emitter.emit(type, event);
  };
  throwIfAborted = () => {
    if (this.aborted) {
      throw new Error(this.reason);
    }
  };
  static abort(reason?: ESObject): AbortSignal {
    const controller = new AbortController();
    controller.abort(reason);
    return controller.signal;
  }
  static timeout(time: number): AbortSignal {
    const controller = new AbortController();
    setTimeout((): void => controller.abort(new Error("TimeoutError")), time);
    return controller.signal;
  }
}

export class AbortController {
  signal: AbortSignal;
  constructor() {
    this.signal = new AbortSignal();
  }
  abort = (reason?: ESObject) => {
    if (this.signal.aborted) return;

    this.signal.aborted = true;

    if (reason) this.signal.reason = reason;
    else this.signal.reason = new Error("AbortError");

    this.signal.dispatchEvent("abort");
  };
  toString = () => {
    return "[object AbortController]";
  };
}
