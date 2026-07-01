import { describe, test } from "../utils/setup.test";
import {
  ByteLengthQueuingStrategy,
  CountQueuingStrategy,
  ReadableStream,
  TransformStream,
  WritableStream,
  collectReadableStream,
  installWebStreams,
  isReadableStream,
  isWritableStream,
} from "../../../third_party/streams/package/index";

function toArray(value: Uint8Array): number[] {
  const bytes: number[] = [];
  for (let index = 0; index < value.length; index += 1) {
    bytes.push(value[index]);
  }
  return bytes;
}

export default function streamTest() {
  describe("StreamsTest", () => {
    test("installs constructors on globalThis for Web Streams validation", (t) => {
      const target: ESObject = {};
      installWebStreams(target);
      t.is(target.ReadableStream, ReadableStream);
      t.is(target.WritableStream, WritableStream);
      t.is(target.TransformStream, TransformStream);
      t.true(new target.ReadableStream() instanceof target.ReadableStream);
      t.true(new target.WritableStream() instanceof target.WritableStream);
    });

    test("reads from a pull based byte stream", async (t) => {
      let index = 0;
      const stream = new ReadableStream<Uint8Array>({
        type: "bytes",
        pull(controller) {
          if (index === 0) {
            controller.enqueue(new Uint8Array([1, 2, 3]));
          } else if (index === 1) {
            controller.enqueue(new Uint8Array([4]));
          } else {
            controller.close();
          }
          index += 1;
        },
      });

      t.true(isReadableStream(stream));
      t.false(stream.locked);
      const reader = stream.getReader();
      t.true(stream.locked);

      const first = await reader.read();
      t.false(first.done);
      t.deepEqual(toArray(first.value as Uint8Array), [1, 2, 3]);

      const second = await reader.read();
      t.false(second.done);
      t.deepEqual(toArray(second.value as Uint8Array), [4]);

      const third = await reader.read();
      t.true(third.done);
      await reader.closed;
      reader.releaseLock();
      t.false(stream.locked);
    });

    test("cancels readable streams through the abort alias", async (t) => {
      let cancelReason = "";
      const stream = new ReadableStream<number>({
        pull(controller) {
          controller.enqueue(1);
        },
        cancel(reason?: ESObject) {
          cancelReason = String(reason);
        },
      });

      await stream.abort("cancelled");
      t.is(cancelReason, "cancelled");
      const reader = stream.getReader();
      const result = await reader.read();
      t.true(result.done);
    });

    test("writes, closes, and aborts writable streams", async (t) => {
      const chunks: number[] = [];
      let closed = false;
      const stream = new WritableStream<number>({
        write(chunk) {
          chunks.push(chunk);
        },
        close() {
          closed = true;
        },
      });

      t.true(isWritableStream(stream));
      const writer = stream.getWriter();
      await writer.ready;
      await writer.write(7);
      await writer.write(8);
      await writer.close();
      await writer.closed;
      writer.releaseLock();

      t.deepEqual(chunks, [7, 8]);
      t.true(closed);

      let aborted = false;
      const abortable = new WritableStream<number>({
        abort(reason?: ESObject) {
          aborted = String(reason) === "stop";
        },
      });
      await abortable.abort("stop");
      t.true(aborted);
    });

    test("transforms chunks through readable and writable sides", async (t) => {
      const stream = new TransformStream<number, string>({
        transform(chunk, controller) {
          controller.enqueue(`value:${chunk}`);
        },
      });
      const writer = stream.writable.getWriter();
      await writer.write(10);
      await writer.write(20);
      await writer.close();
      const chunks = await collectReadableStream(stream.readable);
      t.deepEqual(chunks, ["value:10", "value:20"]);
    });

    test("queuing strategies expose count and byte length sizing", (t) => {
      const count = new CountQueuingStrategy({ highWaterMark: 2 });
      const bytes = new ByteLengthQueuingStrategy({ highWaterMark: 8 });
      t.is(count.highWaterMark, 2);
      t.is(count.size("anything"), 1);
      t.is(bytes.highWaterMark, 8);
      t.is(bytes.size(new Uint8Array([1, 2, 3])), 3);
    });
  });
}
