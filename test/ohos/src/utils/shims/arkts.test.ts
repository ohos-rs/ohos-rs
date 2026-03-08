function getNative(): ESObject {
  return requireNapiPreview("example", true);
}

function encodeUtf8(input: string): Uint8Array {
  const out = [];
  for (let i = 0; i < input.length; i++) {
    const code = input.charCodeAt(i);
    if (code <= 0x7f) {
      out.push(code);
      continue;
    }
    out.push(0x3f);
  }
  return new Uint8Array(out);
}

function decodeUtf8(input: Uint8Array): string {
  let out = "";
  for (const item of input) {
    out += String.fromCharCode(item);
  }
  return out;
}

function wrapBuffer(data: Uint8Array) {
  const value = data as ESObject;
  value.toString = (encoding: string = "utf8") => {
    if (encoding !== "utf8") {
      return decodeUtf8(data);
    }
    return decodeUtf8(data);
  };
  return value;
}

export const buffer = {
  from(data: ESObject) {
    if (typeof data === "string") {
      return wrapBuffer(encodeUtf8(data));
    }
    if (data instanceof ArrayBuffer) {
      return wrapBuffer(new Uint8Array(data));
    }
    if (Array.isArray(data)) {
      return wrapBuffer(new Uint8Array(data));
    }
    if (data instanceof Uint8Array) {
      return wrapBuffer(data);
    }
    return wrapBuffer(new Uint8Array(0));
  },
  alloc(size: number) {
    return wrapBuffer(new Uint8Array(size));
  },
  isBuffer(value: ESObject) {
    return value instanceof Uint8Array || value instanceof ArrayBuffer;
  },
  concat(chunks: Array<Uint8Array>) {
    const total = chunks.reduce((sum, item) => sum + item.length, 0);
    const out = new Uint8Array(total);
    let offset = 0;
    for (const item of chunks) {
      out.set(item, offset);
      offset += item.length;
    }
    return wrapBuffer(out);
  },
};

class ThreadWorker {
  private _onmessage: ((msg: ESObject) => void) | null = null;
  private _onerror: ((err: ESObject) => void) | null = null;
  private _onmessageerror: ((err: ESObject) => void) | null = null;
  private pendingMessages: Array<ESObject> = [];
  name: string;

  constructor(scriptURL: string, options?: ESObject) {
    this.name = options?.name || scriptURL || "mock-worker";
  }

  get onmessage() {
    return this._onmessage;
  }

  set onmessage(handler: ((msg: ESObject) => void) | null) {
    this._onmessage = handler;
    this.flushPendingMessages();
  }

  get onerror() {
    return this._onerror;
  }

  set onerror(handler: ((err: ESObject) => void) | null) {
    this._onerror = handler;
  }

  get onmessageerror() {
    return this._onmessageerror;
  }

  set onmessageerror(handler: ((err: ESObject) => void) | null) {
    this._onmessageerror = handler;
  }

  private flushPendingMessages() {
    if (!this._onmessage || this.pendingMessages.length === 0) {
      return;
    }
    const pending = this.pendingMessages.slice();
    this.pendingMessages = [];
    for (const payload of pending) {
      this._onmessage(payload);
    }
  }

  private emitMessage(data: ESObject) {
    const payload = { data, currentThreadName: this.name };
    if (this._onmessage) {
      this._onmessage(payload);
      return;
    }
    this.pendingMessages.push(payload);
  }

  private emitError(error: ESObject) {
    if (this._onerror) {
      this._onerror(error);
      return;
    }
    if (this._onmessageerror) {
      this._onmessageerror(error);
    }
  }

  private handleMessage(data: ESObject) {
    switch (data?.type) {
      case "require":
        return getNative().Animal.withKind(getNative().Kind.Cat).whoami() + getNative().DEFAULT_COST;
      case "async:buffer":
        return Promise.all(
          Array.from({ length: 100 }).map(() => getNative().bufferPassThrough(buffer.from([1, 2, 3]).buffer)),
        ).then(() => "done");
      case "async:arraybuffer":
        return Promise.all(
          Array.from({ length: 100 }).map(() => getNative().arrayBufferPassThrough(Uint8Array.from([1, 2, 3]))),
        ).then(() => "done");
      case "constructor": {
        let ellie = null;
        for (let i = 0; i < 1000; i++) {
          ellie = new getNative().Animal(getNative().Kind.Cat, "Ellie");
        }
        return ellie.name;
      }
      default:
        throw new TypeError(`Unknown message type: ${data?.type}`);
    }
  }

  postMessage(data: ESObject) {
    try {
      const result = this.handleMessage(data);
      if (result && typeof (result as ESObject).then === "function") {
        (result as Promise<ESObject>)
          .then((value) => {
            this.emitMessage(value);
          })
          .catch((error) => {
            this.emitError(error);
          });
        return;
      }
      this.emitMessage(result);
    } catch (error) {
      this.emitError(error);
    }
  }

  terminate() {
    this.pendingMessages = [];
    return Promise.resolve();
  }
}

export const worker = {
  ThreadWorker,
};
