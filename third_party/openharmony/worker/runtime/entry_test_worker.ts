const { Animal, Kind, DEFAULT_COST, bufferPassThrough, arrayBufferPassThrough } =
  requireNapiPreview("example", true);

export async function handleEntryTestWorker(message: ESObject): Promise<ESObject> {
  switch (message?.type) {
    case "require":
      return Animal.withKind(Kind.Cat).whoami() + DEFAULT_COST;
    case "async:buffer":
      await bufferPassThrough(
        new Uint8Array([104, 101, 108, 108, 111, 32, 119, 111, 114, 107, 101, 114]).buffer,
      );
      return "done";
    case "async:arraybuffer":
      await arrayBufferPassThrough(new Uint8Array([1, 2, 3, 4]));
      return "done";
    case "constructor":
      return new Animal(Kind.Cat, "Ellie").name;
    default:
      throw new Error(`Unsupported worker message type: ${String(message?.type)}`);
  }
}
