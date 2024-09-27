/*
 * Copyright (c) 2021-2024 Huawei Device Co., Ltd.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * define the testcase type : TestType, Size , Level
 */
export const TAG = "[Hypium]";

export const DEFAULT = 0b0000;

export class PrintTag {
  static OHOS_REPORT_WORKER_STATUS = "OHOS_REPORT_WORKER_STATUS";
  static OHOS_REPORT_ALL_RESULT = "OHOS_REPORT_ALL_RESULT";
  static OHOS_REPORT_ALL_CODE = "OHOS_REPORT_ALL_CODE";
  static OHOS_REPORT_ALL_STATUS = "OHOS_REPORT_ALL_STATUS";
  static OHOS_REPORT_RESULT = "OHOS_REPORT_RESULT";
  static OHOS_REPORT_CODE = "OHOS_REPORT_CODE";
  static OHOS_REPORT_STATUS = "OHOS_REPORT_STATUS";
  static OHOS_REPORT_SUM = "OHOS_REPORT_SUM";
  static OHOS_REPORT_STATUS_CODE = "OHOS_REPORT_STATUS_CODE";
}

export class TestType {
  static FUNCTION = 0b1;
  static PERFORMANCE = 0b1 << 1;
  static POWER = 0b1 << 2;
  static RELIABILITY = 0b1 << 3;
  static SECURITY = 0b1 << 4;
  static GLOBAL = 0b1 << 5;
  static COMPATIBILITY = 0b1 << 6;
  static USER = 0b1 << 7;
  static STANDARD = 0b1 << 8;
  static SAFETY = 0b1 << 9;
  static RESILIENCE = 0b1 << 10;
}

export class Size {
  static SMALLTEST = 0b1 << 16;
  static MEDIUMTEST = 0b1 << 17;
  static LARGETEST = 0b1 << 18;
}

export class Level {
  static LEVEL0 = 0b1 << 24;
  static LEVEL1 = 0b1 << 25;
  static LEVEL2 = 0b1 << 26;
  static LEVEL3 = 0b1 << 27;
  static LEVEL4 = 0b1 << 28;
}

export const TESTTYPE = {
  function: 1,
  performance: 1 << 1,
  power: 1 << 2,
  reliability: 1 << 3,
  security: 1 << 4,
  global: 1 << 5,
  compatibility: 1 << 6,
  user: 1 << 7,
  standard: 1 << 8,
  safety: 1 << 9,
  resilience: 1 << 10
};

export const LEVEL = {
  0: 1 << 24,
  1: 1 << 25,
  2: 1 << 26,
  3: 1 << 27,
  4: 1 << 28
};

export const SIZE = {
  small: 1 << 16,
  medium: 1 << 17,
  large: 1 << 18
};

export const KEYSET = [
  "-s class",
  "-s notClass",
  "-s suite",
  "-s itName",
  "-s level",
  "-s testType",
  "-s size",
  "-s timeout",
  "-s dryRun",
  "-s random",
  "-s breakOnError",
  "-s stress",
  "-s coverage",
  "-s skipMessage",
  "-s runSkipped",
  "class",
  "notClass",
  "suite",
  "itName",
  "level",
  "testType",
  "size",
  "timeout",
  "dryRun",
  "random",
  "breakOnError",
  "stress",
  "coverage",
  "skipMessage",
  "runSkipped"
];
