const native = requireNapiPreview("buffer", true);

type BufferLike = Uint8Array & {
  toString: (encoding?: string) => string;
};

function wrap(data: ArrayBuffer | Uint8Array): BufferLike {
  const view = data instanceof Uint8Array ? data : new Uint8Array(data);
  const value = view as BufferLike;
  value.toString = (encoding: string = "utf8") => native.toString(view, encoding);
  return value;
}

const buffer = {
  from(data: ESObject, encoding?: string) {
    if (typeof data === "string") {
      return wrap(native.fromString(data, encoding ?? "utf8"));
    }
    if (data instanceof Uint8Array) {
      return wrap(
        new Uint8Array(data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength)),
      );
    }
    if (data instanceof ArrayBuffer) {
      return wrap(data.slice(0));
    }
    if (Array.isArray(data)) {
      return wrap(new Uint8Array(data as Array<number>));
    }
    return wrap(new Uint8Array(0));
  },

  alloc(size: number) {
    return wrap(native.alloc(size));
  },

  concat(chunks: Array<Uint8Array>) {
    const total = chunks.reduce((sum, item) => sum + item.length, 0);
    const out = new Uint8Array(total);
    let offset = 0;
    for (const item of chunks) {
      out.set(item, offset);
      offset += item.length;
    }
    return wrap(out);
  },

  isBuffer(value: ESObject) {
    return value instanceof Uint8Array || value instanceof ArrayBuffer;
  },
};

export { buffer };
export default { buffer };
