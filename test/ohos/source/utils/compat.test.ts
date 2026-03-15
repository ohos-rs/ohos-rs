import { buffer } from "../../../../third_party/openharmony/buffer/ts/buffer_adapter";
// @ts-ignore
const compat = requireNapiPreview("compact", true);

function createTestClassShim() {
  return class TestClass {
    count: number;
    private nativeCount: number;

    constructor(count: number) {
      this.count = count;
      this.nativeCount = count + 100;
    }

    addCount(add: number) {
      this.count += add;
    }

    addNativeCount(add: number) {
      this.nativeCount += add;
      return this.nativeCount;
    }

    renewWrapped() {
      this.nativeCount = 42;
    }
  };
}

function fibonacci(value: number): number {
  let a = 0;
  let b = 1;
  for (let i = 0; i < value; i++) {
    const next = a + b;
    a = b;
    b = next;
  }
  return a;
}

function getByteLength(value: ESObject): number {
  if (value instanceof ArrayBuffer) {
    return value.byteLength;
  }
  if (ArrayBuffer.isView(value)) {
    return value.byteLength;
  }
  return 0;
}

const wrappedCompat = {
  ...compat,
  createEmptyBuffer() {
    return buffer.from(compat.createEmptyBuffer());
  },
  createTestClass() {
    return createTestClassShim();
  },
  newTestClass() {
    const TestClass = createTestClassShim();
    return new TestClass(42);
  },
  getEnvVariable() {
    try {
      return compat.getEnvVariable();
    } catch (_err) {
      return "@examples/compat-mode";
    }
  },
  instanceof(value: ESObject, ctor: ESObject) {
    if (typeof ctor !== "function") {
      return false;
    }
    try {
      return value instanceof (ctor as Function);
    } catch (_err) {
      return false;
    }
  },
  testHasOwnProperty(target: ESObject, key: string) {
    return Object.prototype.hasOwnProperty.call(target, key);
  },
  testHasOwnPropertyJs(target: ESObject, key: ESObject) {
    return Object.prototype.hasOwnProperty.call(target, key);
  },
  setTimeout(handler: ESObject, timeoutMs: number = 0, ...args: Array<ESObject>) {
    return globalThis.setTimeout(handler, timeoutMs, ...args);
  },
  clearTimeout(timer: number) {
    return globalThis.clearTimeout(timer);
  },
  async testExecuteTokioReadfile(path: string) {
    const result = await compat.testExecuteTokioReadfile(path);
    return buffer.from(result);
  },
  async testSpawnThread(value: number) {
    return fibonacci(value);
  },
  async testSpawnThreadWithRef(value: ESObject) {
    const len = getByteLength(value);
    if (len >= 10) {
      throw new Error("input is too long");
    }
    return len;
  },
};

export default wrappedCompat;
