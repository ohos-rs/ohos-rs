const inMemoryFiles = new Map();

const fs = {
  OpenMode: {
    READ_WRITE: 1,
    CREATE: 2,
  },
  accessSync(path) {
    return inMemoryFiles.has(path);
  },
  unlinkSync(path) {
    inMemoryFiles.delete(path);
  },
  mkdirSync(_path) {},
  openSync(path) {
    if (!inMemoryFiles.has(path)) {
      inMemoryFiles.set(path, "");
    }
    return { fd: path };
  },
  writeSync(fd, content) {
    const current = inMemoryFiles.get(fd) || "";
    inMemoryFiles.set(fd, current + String(content));
    return String(content).length;
  },
  closeSync(_file) {},
};

export default fs;
