const inMemoryFiles = new Map<string, string>();

export const fileIo = {
  AccessModeType: {
    EXIST: 0,
  },
  OpenMode: {
    READ_WRITE: 1,
    CREATE: 2,
  },
  accessSync(path: string) {
    return inMemoryFiles.has(path);
  },
  unlinkSync(path: string) {
    inMemoryFiles.delete(path);
  },
  openSync(path: string) {
    if (!inMemoryFiles.has(path)) {
      inMemoryFiles.set(path, "");
    }
    return { fd: path };
  },
  writeSync(fd: string, content: ESObject) {
    const current = inMemoryFiles.get(fd) || "";
    inMemoryFiles.set(fd, current + String(content));
  },
};
