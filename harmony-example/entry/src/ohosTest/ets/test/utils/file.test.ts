import { fileIo as fs } from "@kit.CoreFileKit";

export const EXAMPLE_STRING = "Hello world!";

export const EXAMPLE_TXT_FILE_NAME = "/test.txt";

export const EXAMPLE_JSON_FILE_NAME = "/test.json";

export const EXAMPLE_JSON = `{
  "license": "MIT",
  "author": "richerfu",
  "name": "@ohos-rs/ada",
  "description": "ada binding for OpenHarmony, powered by ohos-rs",
  "main": "index.ets",
  "version": "0.0.2",
  "types": "libs/index.d.ts",
  "dependencies": {},
}`;

export const writeTxtFile = (path) => {
  const isExist = fs.accessSync(path + "/test.txt", fs.AccessModeType.EXIST);
  if (isExist) {
    fs.unlinkSync(path + "/test.txt");
  }

  let file = fs.openSync(
    path + "/test.txt",
    fs.OpenMode.READ_WRITE | fs.OpenMode.CREATE
  );

  fs.writeSync(file.fd, EXAMPLE_STRING);
};

export const writeJsonFile = (path) => {
  const isExist = fs.accessSync(path + "/test.json", fs.AccessModeType.EXIST);
  if (isExist) {
    fs.unlinkSync(path + "/test.json");
  }
  let file = fs.openSync(
    path + "/test.json",
    fs.OpenMode.READ_WRITE | fs.OpenMode.CREATE
  );
  fs.writeSync(file.fd, EXAMPLE_JSON);
};
