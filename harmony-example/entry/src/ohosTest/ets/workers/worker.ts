import {
  buffer,
  ErrorEvent,
  MessageEvents,
  ThreadWorkerGlobalScope,
  worker
} from "@kit.ArkTS";
import * as native from "libexample.so";

const workerPort: ThreadWorkerGlobalScope = worker.workerPort;

/**
 * Defines the event handler to be called when the worker thread receives a message sent by the host thread.
 * The event handler is executed in the worker thread.
 *
 * @param e message data
 */
workerPort.onmessage = (e: MessageEvents) => {
  switch (e.type) {
    case "require":
      workerPort.postMessage(
        native.Animal.withKind(native.Kind.Cat).whoami() + native.DEFAULT_COST
      );
      break;
    case "async:buffer":
      Promise.all(
        Array.from({ length: 100 }).map(() =>
          native.bufferPassThrough(buffer.from([1, 2, 3]).buffer)
        )
      )
        .then(() => {
          workerPort.postMessage("done");
        })
        .catch((e) => {
          throw e;
        });
      break;
    case "async:arraybuffer":
      Promise.all(
        Array.from({ length: 100 }).map(() =>
          native.arrayBufferPassThrough(Uint8Array.from([1, 2, 3]))
        )
      )
        .then(() => {
          workerPort.postMessage("done");
        })
        .catch((e) => {
          throw e;
        });

      break;
    case "constructor":
      let ellie;
      for (let i = 0; i < 1000; i++) {
        ellie = new native.Animal(native.Kind.Cat, "Ellie");
      }
      workerPort.postMessage(ellie.name);
      break;
    default:
      throw new TypeError(`Unknown message type: ${e.type}`);
  }
};

/**
 * Defines the event handler to be called when the worker receives a message that cannot be deserialized.
 * The event handler is executed in the worker thread.
 *
 * @param e message data
 */
workerPort.onmessageerror = (e: MessageEvents) => {};

/**
 * Defines the event handler to be called when an exception occurs during worker execution.
 * The event handler is executed in the worker thread.
 *
 * @param e error message
 */
workerPort.onerror = (e: ErrorEvent) => {};
