type MaybePromise<T> = T | Promise<T>;
type Resolve<T> = (value: T | PromiseLike<T>) => void;
type Reject = (reason?: ESObject) => void;

export class ReadableStreamReadResult<T> {
  done: boolean;
  value?: T;

  constructor(done: boolean, value?: T) {
    this.done = done;
    if (value !== undefined) {
      this.value = value;
    }
  }
}

export interface UnderlyingSource<T> {
  type?: string;
  start?(controller: ReadableStreamDefaultController<T>): MaybePromise<void>;
  pull?(controller: ReadableStreamDefaultController<T>): MaybePromise<void>;
  cancel?(reason?: ESObject): MaybePromise<void>;
}

export interface UnderlyingSink<T> {
  start?(controller: WritableStreamDefaultController): MaybePromise<void>;
  write?(chunk: T, controller: WritableStreamDefaultController): MaybePromise<void>;
  close?(): MaybePromise<void>;
  abort?(reason?: ESObject): MaybePromise<void>;
}

export interface Transformer<I, O> {
  start?(controller: TransformStreamDefaultController<O>): MaybePromise<void>;
  transform?(chunk: I, controller: TransformStreamDefaultController<O>): MaybePromise<void>;
  flush?(controller: TransformStreamDefaultController<O>): MaybePromise<void>;
  cancel?(reason?: ESObject): MaybePromise<void>;
}

export interface QueuingStrategy<T> {
  highWaterMark?: number;
  size?(chunk: T): number;
}

class EmptyUnderlyingSource<T> implements UnderlyingSource<T> {}

class EmptyUnderlyingSink<T> implements UnderlyingSink<T> {}

class EmptyTransformer<I, O> implements Transformer<I, O> {}

class DefaultQueuingStrategy<T> implements QueuingStrategy<T> {
  highWaterMark: number = 1;

  size(_chunk: T): number {
    return 1;
  }
}

class ReadRequest<T> {
  resolve: Resolve<ReadableStreamReadResult<T>>;
  reject: Reject;

  constructor(resolve: Resolve<ReadableStreamReadResult<T>>, reject: Reject) {
    this.resolve = resolve;
    this.reject = reject;
  }
}

class ControllerHolder<T> {
  controller?: ReadableStreamDefaultController<T>;
}

export class ReadableWritablePair<I, O> {
  readable: ReadableStream<O>;
  writable: WritableStream<I>;

  constructor(readable: ReadableStream<O>, writable: WritableStream<I>) {
    this.readable = readable;
    this.writable = writable;
  }
}

function readResult<T>(done: boolean, value?: T): ReadableStreamReadResult<T> {
  return new ReadableStreamReadResult<T>(done, value);
}

function promiseCall<T>(fn: (() => MaybePromise<T>) | undefined): Promise<T | undefined> {
  if (!fn) {
    return Promise.resolve(undefined);
  }
  try {
    return Promise.resolve(fn());
  } catch (error) {
    return Promise.reject(error);
  }
}

function isPromiseLike(value: ESObject): boolean {
  return value !== null && value !== undefined && typeof value.then === "function";
}

function defaultSize(_chunk: ESObject): number {
  return 1;
}

export class CountQueuingStrategy {
  readonly highWaterMark: number;

  constructor(init: QueuingStrategy<ESObject>) {
    this.highWaterMark = Number(init.highWaterMark ?? 1);
  }

  size(_chunk: ESObject): number {
    return 1;
  }
}

export class ByteLengthQueuingStrategy {
  readonly highWaterMark: number;

  constructor(init: QueuingStrategy<ESObject>) {
    this.highWaterMark = Number(init.highWaterMark ?? 0);
  }

  size(chunk: ESObject): number {
    if (chunk === null || chunk === undefined) {
      return 0;
    }
    if (typeof chunk.byteLength === "number") {
      return Number(chunk.byteLength);
    }
    if (typeof chunk.length === "number") {
      return Number(chunk.length);
    }
    return 1;
  }
}

export class ReadableStreamDefaultController<T> {
  private stream: ReadableStream<T>;

  constructor(stream: ReadableStream<T>) {
    this.stream = stream;
  }

  get desiredSize(): number {
    return this.stream.desiredSize();
  }

  enqueue(chunk: T): void {
    this.stream.enqueue(chunk);
  }

  close(): void {
    this.stream.closeReadable();
  }

  error(reason?: ESObject): void {
    this.stream.errorReadable(reason);
  }
}

export class ReadableStreamDefaultReader<T> {
  private stream: ReadableStream<T> | null;
  private closedPromise: Promise<void>;
  private closedResolve: Resolve<void> = () => {};
  private closedReject: Reject = () => {};

  constructor(stream: ReadableStream<T>) {
    if (stream.locked) {
      throw new TypeError("ReadableStream is already locked");
    }
    this.stream = stream;
    stream.reader = this;
    this.closedPromise = new Promise<void>((resolve, reject) => {
      this.closedResolve = resolve;
      this.closedReject = reject;
    });
    stream.attachReader(this);
  }

  get closed(): Promise<void> {
    return this.closedPromise;
  }

  read(): Promise<ReadableStreamReadResult<T>> {
    if (!this.stream) {
      return Promise.reject(new TypeError("Reader lock has been released"));
    }
    return this.stream.read();
  }

  cancel(reason?: ESObject): Promise<void> {
    if (!this.stream) {
      return Promise.reject(new TypeError("Reader lock has been released"));
    }
    return this.stream.cancel(reason);
  }

  releaseLock(): void {
    if (!this.stream) {
      return;
    }
    this.stream.reader = null;
    this.stream = null;
  }

  notifyClosed(): void {
    this.closedResolve(undefined);
  }

  notifyErrored(reason?: ESObject): void {
    this.closedReject(reason);
  }
}

export class ReadableStream<T = ESObject> {
  private source: UnderlyingSource<T>;
  private controller: ReadableStreamDefaultController<T>;
  private queue: T[] = [];
  private readRequests: ReadRequest<T>[] = [];
  private closeRequested = false;
  private disturbed = false;
  private errored: ESObject | null = null;
  private pulling = false;
  private highWaterMark: number;
  private sizeAlgorithm: (chunk: ESObject) => number;

  reader: ReadableStreamDefaultReader<T> | null = null;

  constructor(source?: UnderlyingSource<T>, strategy?: QueuingStrategy<T>) {
    const effectiveSource = source ?? new EmptyUnderlyingSource<T>();
    const effectiveStrategy = strategy ?? new DefaultQueuingStrategy<T>();
    this.source = effectiveSource;
    this.controller = new ReadableStreamDefaultController<T>(this);
    this.highWaterMark = Number(effectiveStrategy.highWaterMark ?? 1);
    this.sizeAlgorithm =
      (effectiveStrategy.size as ((chunk: ESObject) => number) | undefined) ?? defaultSize;

    promiseCall(() => this.source.start?.(this.controller)).catch((error: ESObject) => {
      this.errorReadable(error);
    });
  }

  get locked(): boolean {
    return this.reader !== null;
  }

  getReader(): ReadableStreamDefaultReader<T> {
    return new ReadableStreamDefaultReader<T>(this);
  }

  cancel(reason?: ESObject): Promise<void> {
    if (this.closeRequested) {
      return Promise.resolve();
    }
    this.queue = [];
    this.closeReadable();
    return promiseCall(() => this.source.cancel?.(reason)).then(() => undefined);
  }

  abort(reason?: ESObject): Promise<void> {
    return this.cancel(reason);
  }

  pipeThrough<O>(transform: ReadableWritablePair<T, O>): ReadableStream<O> {
    this.pipeTo(transform.writable).catch((error: ESObject) => {
      transform.readable.cancel(error);
    });
    return transform.readable;
  }

  pipeTo(destination: WritableStream<T>): Promise<void> {
    const reader = this.getReader();
    const writer = destination.getWriter();
    const pump = (): Promise<void> => {
      return reader.read().then((result) => {
        if (result.done) {
          return writer.close();
        }
        return writer.write(result.value as T).then(pump);
      });
    };
    return pump().finally(() => {
      reader.releaseLock();
      writer.releaseLock();
    });
  }

  tee(): ReadableStream<T>[] {
    const leftControllerHolder = new ControllerHolder<T>();
    const rightControllerHolder = new ControllerHolder<T>();
    const left = new ReadableStream<T>(new TeeSource<T>(leftControllerHolder));
    const right = new ReadableStream<T>(new TeeSource<T>(rightControllerHolder));
    const reader = this.getReader();
    const pump = (): void => {
      reader
        .read()
        .then((result) => {
          if (result.done) {
            leftControllerHolder.controller?.close();
            rightControllerHolder.controller?.close();
            return;
          }
          leftControllerHolder.controller?.enqueue(result.value as T);
          rightControllerHolder.controller?.enqueue(result.value as T);
          pump();
        })
        .catch((error: ESObject) => {
          leftControllerHolder.controller?.error(error);
          rightControllerHolder.controller?.error(error);
        });
    };
    pump();
    return [left, right];
  }

  desiredSize(): number {
    let total = 0;
    for (const chunk of this.queue) {
      total += this.sizeAlgorithm(chunk as ESObject);
    }
    return this.highWaterMark - total;
  }

  attachReader(reader: ReadableStreamDefaultReader<T>): void {
    if (this.closeRequested) {
      reader.notifyClosed();
    }
    if (this.errored !== null) {
      reader.notifyErrored(this.errored);
    }
  }

  read(): Promise<ReadableStreamReadResult<T>> {
    this.disturbed = true;
    if (this.errored !== null) {
      return Promise.reject(this.errored);
    }
    if (this.queue.length > 0) {
      const value = this.queue.shift() as T;
      this.pullIfNeeded();
      return Promise.resolve(readResult<T>(false, value));
    }
    if (this.closeRequested) {
      return Promise.resolve(readResult<T>(true));
    }
    const promise = new Promise<ReadableStreamReadResult<T>>((resolve, reject) => {
      this.readRequests.push(new ReadRequest<T>(resolve, reject));
    });
    this.pullIfNeeded();
    return promise;
  }

  enqueue(chunk: T): void {
    if (this.closeRequested || this.errored !== null) {
      throw new TypeError("ReadableStream is not readable");
    }
    const request = this.readRequests.shift();
    if (request) {
      request.resolve(readResult<T>(false, chunk));
      return;
    }
    this.queue.push(chunk);
  }

  closeReadable(): void {
    if (this.closeRequested) {
      return;
    }
    this.closeRequested = true;
    while (this.readRequests.length > 0) {
      const request = this.readRequests.shift() as ReadRequest<T>;
      request.resolve(readResult<T>(true));
    }
    this.reader?.notifyClosed();
  }

  errorReadable(reason?: ESObject): void {
    if (this.errored !== null) {
      return;
    }
    this.errored = reason ?? new Error("ReadableStream error");
    while (this.readRequests.length > 0) {
      const request = this.readRequests.shift() as ReadRequest<T>;
      request.reject(this.errored);
    }
    this.reader?.notifyErrored(this.errored);
  }

  private pullIfNeeded(): void {
    if (this.pulling || this.closeRequested || this.errored !== null) {
      return;
    }
    if (!this.source.pull) {
      return;
    }
    this.pulling = true;
    try {
      const result = this.source.pull(this.controller);
      if (!isPromiseLike(result as ESObject)) {
        this.pulling = false;
        return;
      }
      Promise.resolve(result)
        .catch((error: ESObject) => {
          this.errorReadable(error);
        })
        .finally(() => {
          this.pulling = false;
          if (this.readRequests.length > 0 && this.queue.length === 0) {
            this.pullIfNeeded();
          }
        });
    } catch (error) {
      this.pulling = false;
      this.errorReadable(error);
    }
  }
}

class TeeSource<T> implements UnderlyingSource<T> {
  private holder: ControllerHolder<T>;

  constructor(holder: ControllerHolder<T>) {
    this.holder = holder;
  }

  start(controller: ReadableStreamDefaultController<T>): void {
    this.holder.controller = controller;
  }
}

export class WritableStreamDefaultController {
  private stream: WritableStream<ESObject>;

  constructor(stream: WritableStream<ESObject>) {
    this.stream = stream;
  }

  error(reason?: ESObject): void {
    this.stream.errorWritable(reason);
  }
}

export class WritableStreamDefaultWriter<T = ESObject> {
  private stream: WritableStream<T> | null;

  constructor(stream: WritableStream<T>) {
    if (stream.locked) {
      throw new TypeError("WritableStream is already locked");
    }
    this.stream = stream;
    stream.writer = this as WritableStreamDefaultWriter<ESObject>;
  }

  get closed(): Promise<void> {
    return this.stream
      ? this.stream.closed
      : Promise.reject(new TypeError("Writer lock has been released"));
  }

  get ready(): Promise<void> {
    return this.stream
      ? this.stream.ready
      : Promise.reject(new TypeError("Writer lock has been released"));
  }

  write(chunk: T): Promise<void> {
    if (!this.stream) {
      return Promise.reject(new TypeError("Writer lock has been released"));
    }
    return this.stream.write(chunk);
  }

  close(): Promise<void> {
    if (!this.stream) {
      return Promise.reject(new TypeError("Writer lock has been released"));
    }
    return this.stream.close();
  }

  abort(reason?: ESObject): Promise<void> {
    if (!this.stream) {
      return Promise.reject(new TypeError("Writer lock has been released"));
    }
    return this.stream.abort(reason);
  }

  releaseLock(): void {
    if (!this.stream) {
      return;
    }
    this.stream.writer = null;
    this.stream = null;
  }
}

export class WritableStream<T = ESObject> {
  private sink: UnderlyingSink<T>;
  private controller: WritableStreamDefaultController;
  private closedResolve: Resolve<void> = () => {};
  private closedReject: Reject = () => {};
  private errored: ESObject | null = null;
  private closeRequested = false;

  readonly ready: Promise<void> = Promise.resolve();
  readonly closed: Promise<void>;
  writer: WritableStreamDefaultWriter<ESObject> | null = null;

  constructor(sink?: UnderlyingSink<T>, _strategy?: QueuingStrategy<T>) {
    this.sink = sink ?? new EmptyUnderlyingSink<T>();
    this.controller = new WritableStreamDefaultController(this as WritableStream<ESObject>);
    this.closed = new Promise<void>((resolve, reject) => {
      this.closedResolve = resolve;
      this.closedReject = reject;
    });
    promiseCall(() => this.sink.start?.(this.controller)).catch((error: ESObject) => {
      this.errorWritable(error);
    });
  }

  get locked(): boolean {
    return this.writer !== null;
  }

  getWriter(): WritableStreamDefaultWriter<T> {
    return new WritableStreamDefaultWriter<T>(this);
  }

  write(chunk: T): Promise<void> {
    if (this.errored !== null) {
      return Promise.reject(this.errored);
    }
    if (this.closeRequested) {
      return Promise.reject(new TypeError("WritableStream is closed"));
    }
    return promiseCall(() => this.sink.write?.(chunk, this.controller)).then(() => undefined);
  }

  close(): Promise<void> {
    if (this.errored !== null) {
      return Promise.reject(this.errored);
    }
    if (this.closeRequested) {
      return Promise.resolve();
    }
    this.closeRequested = true;
    return promiseCall(() => this.sink.close?.()).then(() => {
      this.closedResolve(undefined);
    });
  }

  abort(reason?: ESObject): Promise<void> {
    if (this.closeRequested) {
      return Promise.resolve();
    }
    this.closeRequested = true;
    return promiseCall(() => this.sink.abort?.(reason)).then(() => {
      this.closedResolve(undefined);
    });
  }

  errorWritable(reason?: ESObject): void {
    if (this.errored !== null) {
      return;
    }
    this.errored = reason ?? new Error("WritableStream error");
    this.closedReject(this.errored);
  }
}

export class TransformStreamDefaultController<T> {
  private readableController: ReadableStreamDefaultController<T>;
  private terminateCallback: () => void;

  constructor(
    readableController: ReadableStreamDefaultController<T>,
    terminateCallback: () => void,
  ) {
    this.readableController = readableController;
    this.terminateCallback = terminateCallback;
  }

  get desiredSize(): number {
    return this.readableController.desiredSize;
  }

  enqueue(chunk: T): void {
    this.readableController.enqueue(chunk);
  }

  error(reason?: ESObject): void {
    this.readableController.error(reason);
  }

  terminate(): void {
    this.terminateCallback();
  }
}

class TransformState<O> {
  readableController?: ReadableStreamDefaultController<O>;
  transformController?: TransformStreamDefaultController<O>;
}

class TransformReadableSource<I, O> implements UnderlyingSource<O> {
  private transformer: Transformer<I, O>;
  private state: TransformState<O>;

  constructor(transformer: Transformer<I, O>, state: TransformState<O>) {
    this.transformer = transformer;
    this.state = state;
  }

  start(controller: ReadableStreamDefaultController<O>): MaybePromise<void> {
    this.state.readableController = controller;
    this.state.transformController = new TransformStreamDefaultController<O>(controller, () => {
      controller.close();
    });
    return this.transformer.start?.(this.state.transformController);
  }

  cancel(reason?: ESObject): MaybePromise<void> {
    return this.transformer.cancel?.(reason);
  }
}

class TransformWritableSink<I, O> implements UnderlyingSink<I> {
  private transformer: Transformer<I, O>;
  private state: TransformState<O>;

  constructor(transformer: Transformer<I, O>, state: TransformState<O>) {
    this.transformer = transformer;
    this.state = state;
  }

  write(chunk: I, _controller: WritableStreamDefaultController): MaybePromise<void> {
    const transformController = this.state.transformController;
    if (transformController === undefined) {
      throw new TypeError("TransformStream is not started");
    }
    if (this.transformer.transform) {
      return this.transformer.transform(chunk, transformController);
    }

    const readableController = this.state.readableController;
    if (readableController === undefined) {
      throw new TypeError("TransformStream is not readable");
    }
    readableController.enqueue(chunk as ESObject as O);
    return undefined;
  }

  close(): MaybePromise<void> {
    const transformController = this.state.transformController;
    const readableController = this.state.readableController;
    if (transformController === undefined || readableController === undefined) {
      throw new TypeError("TransformStream is not started");
    }
    return promiseCall(() => this.transformer.flush?.(transformController)).then(() => {
      readableController.close();
    });
  }

  abort(reason?: ESObject): void {
    const readableController = this.state.readableController;
    if (readableController !== undefined) {
      readableController.error(reason);
    }
  }
}

export class TransformStream<I = ESObject, O = I> {
  readonly readable: ReadableStream<O>;
  readonly writable: WritableStream<I>;

  constructor(
    transformer?: Transformer<I, O>,
    writableStrategy?: QueuingStrategy<I>,
    readableStrategy?: QueuingStrategy<O>,
  ) {
    const effectiveTransformer = transformer ?? new EmptyTransformer<I, O>();
    const state = new TransformState<O>();
    this.readable = new ReadableStream<O>(
      new TransformReadableSource<I, O>(effectiveTransformer, state),
      readableStrategy,
    );
    this.writable = new WritableStream<I>(
      new TransformWritableSink<I, O>(effectiveTransformer, state),
      writableStrategy,
    );
  }
}

export function installWebStreams(target: ESObject = globalThis as ESObject): void {
  target.ReadableStream = target.ReadableStream ?? ReadableStream;
  target.ReadableStreamDefaultReader =
    target.ReadableStreamDefaultReader ?? ReadableStreamDefaultReader;
  target.ReadableStreamDefaultController =
    target.ReadableStreamDefaultController ?? ReadableStreamDefaultController;
  target.WritableStream = target.WritableStream ?? WritableStream;
  target.WritableStreamDefaultWriter =
    target.WritableStreamDefaultWriter ?? WritableStreamDefaultWriter;
  target.WritableStreamDefaultController =
    target.WritableStreamDefaultController ?? WritableStreamDefaultController;
  target.TransformStream = target.TransformStream ?? TransformStream;
  target.TransformStreamDefaultController =
    target.TransformStreamDefaultController ?? TransformStreamDefaultController;
  target.ByteLengthQueuingStrategy = target.ByteLengthQueuingStrategy ?? ByteLengthQueuingStrategy;
  target.CountQueuingStrategy = target.CountQueuingStrategy ?? CountQueuingStrategy;
}

export function isReadableStream(value: ESObject): boolean {
  return value instanceof ReadableStream;
}

export function isWritableStream(value: ESObject): boolean {
  return value instanceof WritableStream;
}

export async function collectReadableStream<T>(stream: ReadableStream<T>): Promise<T[]> {
  const reader = stream.getReader();
  const chunks: T[] = [];
  try {
    while (true) {
      const result = await reader.read();
      if (result.done) {
        return chunks;
      }
      chunks.push(result.value as T);
    }
  } finally {
    reader.releaseLock();
  }
}

installWebStreams();
