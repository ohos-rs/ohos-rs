// Web Streams are defined here to prevent a dependency on the `dom` library,
// which is not available in the ArkTS type environment.
pub const STREAM_TS: &str = r#"export type ArrayBufferView =
  | Int8Array
  | Uint8Array
  | Uint8ClampedArray
  | Int16Array
  | Uint16Array
  | Int32Array
  | Uint32Array
  | Float32Array
  | Float64Array
  | BigInt64Array
  | BigUint64Array
  | DataView;

export interface PromiseLike<T> {
  then<TResult1 = T, TResult2 = never>(
    onfulfilled?: ((value: T) => TResult1 | PromiseLike<TResult1>) | null,
    onrejected?: ((reason: any) => TResult2 | PromiseLike<TResult2>) | null
  ): PromiseLike<TResult1 | TResult2>;
}

export interface IteratorYieldResult<TYield> {
  done?: false;
  value: TYield;
}

export interface IteratorReturnResult<TReturn> {
  done: true;
  value: TReturn;
}

export type IteratorResult<T, TReturn = any> =
  | IteratorYieldResult<T>
  | IteratorReturnResult<TReturn>;

export interface AsyncIterator<T, TReturn = any, TNext = any> {
  next(value?: TNext): Promise<IteratorResult<T, TReturn>>;
  return?(value?: TReturn | PromiseLike<TReturn>): Promise<IteratorResult<T, TReturn>>;
  throw?(e?: any): Promise<IteratorResult<T, TReturn>>;
}

export interface AsyncIterable<T> {
  [Symbol.asyncIterator](): AsyncIterator<T>;
}

export interface AsyncIterableIterator<T> extends AsyncIterator<T>, AsyncIterable<T> {
  [Symbol.asyncIterator](): AsyncIterableIterator<T>;
}

export interface QueuingStrategy<T = any> {
  highWaterMark?: number;
  size?: QueuingStrategySize<T>;
}

export interface QueuingStrategyInit {
  highWaterMark: number;
}

export type QueuingStrategySize<T = any> = (chunk?: T) => number;

export interface StreamPipeOptions {
  preventAbort?: boolean;
  preventCancel?: boolean;
  preventClose?: boolean;
  signal?: AbortSignal;
}

export interface ReadableStreamIteratorOptions {
  preventCancel?: boolean;
}

export interface ReadableWritablePair<R = any, W = any> {
  readable: ReadableStream<R>;
  writable: WritableStream<W>;
}

export type ReadableStreamReader<T = any> =
  | ReadableStreamDefaultReader<T>
  | ReadableStreamBYOBReader;

export type ReadableStreamController<T = any> =
  | ReadableStreamDefaultController<T>
  | ReadableByteStreamController;

export interface ReadableStreamReadValueResult<T> {
  done: false;
  value: T;
}

export interface ReadableStreamReadDoneResult<T> {
  done: true;
  value?: T;
}

export type ReadableStreamReadResult<T> =
  | ReadableStreamReadValueResult<T>
  | ReadableStreamReadDoneResult<T>;

export interface ReadableStreamBYOBReadDoneResult {
  done: true;
  value?: ArrayBufferView;
}

export interface ReadableStreamBYOBReadValueResult {
  done: false;
  value: ArrayBufferView;
}

export type ReadableStreamBYOBReadResult =
  | ReadableStreamBYOBReadValueResult
  | ReadableStreamBYOBReadDoneResult;

export interface ReadableStreamGetReaderOptions {
  mode?: "byob";
}

export interface ReadableStreamBYOBRequest {
  readonly view: ArrayBufferView | null;
  respond(bytesWritten: number): void;
  respondWithNewView(view: ArrayBufferView): void;
}

export interface ReadableByteStreamController {
  readonly byobRequest: ReadableStreamBYOBRequest | null;
  readonly desiredSize: number | null;
  close(): void;
  enqueue(chunk: ArrayBufferView): void;
  error(e?: any): void;
}

export interface ReadableStreamDefaultController<R = any> {
  readonly desiredSize: number | null;
  close(): void;
  enqueue(chunk?: R): void;
  error(e?: any): void;
}

export interface ReadableStreamDefaultReader<R = any> {
  readonly closed: Promise<void>;
  cancel(reason?: any): Promise<void>;
  read(): Promise<ReadableStreamReadResult<R>>;
  releaseLock(): void;
}

export interface ReadableStreamBYOBReader {
  readonly closed: Promise<void>;
  cancel(reason?: any): Promise<void>;
  read(view: ArrayBufferView): Promise<ReadableStreamBYOBReadResult>;
  releaseLock(): void;
}

export interface UnderlyingSource<R = any> {
  autoAllocateChunkSize?: number;
  cancel?: UnderlyingSourceCancelCallback;
  pull?: UnderlyingSourcePullCallback<R>;
  start?: UnderlyingSourceStartCallback<R>;
  type?: ReadableStreamType;
}

export type UnderlyingSourceCancelCallback = (reason?: any) => void | PromiseLike<void>;

export type UnderlyingSourcePullCallback<R = any> = (
  controller: ReadableStreamController<R>
) => void | PromiseLike<void>;

export type UnderlyingSourceStartCallback<R = any> = (
  controller: ReadableStreamController<R>
) => any;

export type ReadableStreamType = "bytes";

export class ReadableStream<R = any> implements AsyncIterable<R> {
  readonly locked: boolean;
  constructor(underlyingSource?: UnderlyingSource<R>, strategy?: QueuingStrategy<R>);
  cancel(reason?: any): Promise<void>;
  getReader(): ReadableStreamDefaultReader<R>;
  getReader(options: { mode: "byob" }): ReadableStreamBYOBReader;
  getReader(options?: ReadableStreamGetReaderOptions): ReadableStreamReader<R>;
  pipeThrough<T>(
    transform: ReadableWritablePair<T, R>,
    options?: StreamPipeOptions
  ): ReadableStream<T>;
  pipeTo(destination: WritableStream<R>, options?: StreamPipeOptions): Promise<void>;
  tee(): [ReadableStream<R>, ReadableStream<R>];
  values(options?: ReadableStreamIteratorOptions): AsyncIterableIterator<R>;
  [Symbol.asyncIterator](options?: ReadableStreamIteratorOptions): AsyncIterableIterator<R>;
}

export interface WritableStreamDefaultController {
  readonly signal: AbortSignal;
  error(e?: any): void;
}

export interface WritableStreamDefaultWriter<W = any> {
  readonly closed: Promise<void>;
  readonly desiredSize: number | null;
  readonly ready: Promise<void>;
  abort(reason?: any): Promise<void>;
  close(): Promise<void>;
  releaseLock(): void;
  write(chunk?: W): Promise<void>;
}

export interface UnderlyingSink<W = any> {
  abort?: UnderlyingSinkAbortCallback;
  close?: UnderlyingSinkCloseCallback;
  start?: UnderlyingSinkStartCallback;
  type?: undefined;
  write?: UnderlyingSinkWriteCallback<W>;
}

export type UnderlyingSinkAbortCallback = (reason?: any) => void | PromiseLike<void>;

export type UnderlyingSinkCloseCallback = () => void | PromiseLike<void>;

export type UnderlyingSinkStartCallback = (controller: WritableStreamDefaultController) => any;

export type UnderlyingSinkWriteCallback<W = any> = (
  chunk: W,
  controller: WritableStreamDefaultController
) => void | PromiseLike<void>;

export class WritableStream<W = any> {
  readonly locked: boolean;
  constructor(underlyingSink?: UnderlyingSink<W>, strategy?: QueuingStrategy<W>);
  abort(reason?: any): Promise<void>;
  close(): Promise<void>;
  getWriter(): WritableStreamDefaultWriter<W>;
}

export interface TransformStreamDefaultController<O = any> {
  readonly desiredSize: number | null;
  enqueue(chunk?: O): void;
  error(reason?: any): void;
  terminate(): void;
}

export interface Transformer<I = any, O = any> {
  flush?: TransformerFlushCallback<O>;
  readableType?: undefined;
  start?: TransformerStartCallback<O>;
  transform?: TransformerTransformCallback<I, O>;
  writableType?: undefined;
}

export type TransformerFlushCallback<O = any> = (
  controller: TransformStreamDefaultController<O>
) => void | PromiseLike<void>;

export type TransformerStartCallback<O = any> = (
  controller: TransformStreamDefaultController<O>
) => any;

export type TransformerTransformCallback<I = any, O = any> = (
  chunk: I,
  controller: TransformStreamDefaultController<O>
) => void | PromiseLike<void>;

export class TransformStream<I = any, O = any> {
  readonly readable: ReadableStream<O>;
  readonly writable: WritableStream<I>;
  constructor(
    transformer?: Transformer<I, O>,
    writableStrategy?: QueuingStrategy<I>,
    readableStrategy?: QueuingStrategy<O>
  );
}

export class ByteLengthQueuingStrategy implements QueuingStrategy<ArrayBufferView> {
  readonly highWaterMark: number;
  constructor(init: QueuingStrategyInit);
  size(chunk?: ArrayBufferView): number;
}

export class CountQueuingStrategy implements QueuingStrategy<any> {
  readonly highWaterMark: number;
  constructor(init: QueuingStrategyInit);
  size(chunk?: any): number;
}

export type WriteableStream<W = any> = WritableStream<W>;
"#;
