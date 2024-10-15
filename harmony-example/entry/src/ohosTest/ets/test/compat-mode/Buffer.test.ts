import { describe, test } from "../utils/setup.test";
import bindings from "../utils/compat.test";
import { buffer } from "@kit.ArkTS";

export default () => {
  describe("CompatBufferTest", () => {
    test("should get buffer length", (t) => {
      const fixture = buffer.from("wow, hello");
      t.is(bindings.getBufferLength(fixture.buffer), fixture.length);
    });

    test("should stringify buffer", (t) => {
      const fixture = "wow, hello";
      t.is(bindings.bufferToString(buffer.from(fixture).buffer), fixture);
    });

    test("should copy", (t) => {
      const fixture = buffer.from("wow, hello");
      const copyBuffer = bindings.copyBuffer(fixture.buffer);
      t.deepEqual(copyBuffer, fixture.buffer);
      t.not(fixture.buffer, copyBuffer);
    });

    test("should create borrowed buffer with noop finalize", (t) => {
      t.deepEqual(
        bindings.createBorrowedBufferWithNoopFinalize(),
        buffer.from([1, 2, 3]).buffer
      );
    });

    test("should create borrowed buffer with finalize", (t) => {
      t.deepEqual(
        bindings.createBorrowedBufferWithFinalize(),
        buffer.from([1, 2, 3]).buffer
      );
    });

    test("should create empty borrowed buffer with finalize", (t) => {
      t.throws(
        () => bindings.createEmptyBorrowedBufferWithFinalize().toString(),
        {
          message: "Borrowed data should not be null"
        }
      );
      t.throws(
        () => bindings.createEmptyBorrowedBufferWithFinalize().toString(),
        {
          message: "Borrowed data should not be null"
        }
      );
    });

    test("should create empty buffer", (t) => {
      t.is(bindings.createEmptyBuffer().toString(), "");
      t.is(bindings.createEmptyBuffer().toString(), "");
    });

    test("should be able to mutate buffer", (t) => {
      const fixture = buffer.from([0, 1]).buffer;
      bindings.mutateBuffer(fixture);
      const buf = buffer.from(fixture);
      t.is(buf[1], 42);
    });
  });
};
