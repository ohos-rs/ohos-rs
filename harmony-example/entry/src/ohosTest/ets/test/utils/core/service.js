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

import SysTestKit from "./module/kit/SysTestKit";
import { TAG } from "./Constant";
import LogExpectError from "./module/report/LogExpectError";
import { NestFilter } from "./module/config/Filter";

function assertTrueFun(actualValue) {
  let result = {
    pass: actualValue === true,
    message: "expect true, actualValue is " + actualValue
  };
  return result;
}

function assertEqualFun(actualValue, args) {
  let msg = "expect " + actualValue + " equals " + args[0];
  if (actualValue === args[0]) {
    // 数值相同,提示数据类型
    const aClassName = Object.prototype.toString.call(actualValue);
    const bClassName = Object.prototype.toString.call(args[0]);
    msg =
      "expect " +
      actualValue +
      aClassName +
      " equals " +
      args[0] +
      bClassName +
      "strict mode inspect type";
  }
  let result = {
    pass: actualValue === args[0],
    expectValue: args[0],
    message: msg
  };
  return result;
}

function assertThrowFun(actual, args) {
  const result = {
    pass: false
  };
  if (typeof actual !== "function") {
    result.message = "toThrow's Actual should be a Function";
  } else {
    let hasThrow = false;
    let throwError;
    try {
      actual();
    } catch (e) {
      hasThrow = true;
      throwError = e;
    }
    if (!hasThrow) {
      result.message = "function did not throw an exception";
    } else if (throwError && throwError.message === args[0]) {
      result.pass = true;
    } else {
      result.message = `expect to throw ${args[0]} , actual throw ${throwError.message}`;
    }
  }
  return result;
}

class AssertException extends Error {
  constructor(message) {
    super();
    this.name = "AssertException";
    this.message = message;
  }
}

function getFuncWithArgsZero(func, timeout, isStressTest) {
  return new Promise(async (resolve, reject) => {
    let timer = null;
    if (!isStressTest) {
      timer = setTimeout(() => {
        reject(new Error("execute timeout " + timeout + "ms"));
      }, timeout);
    }
    try {
      await func();
    } catch (err) {
      reject(err);
    }
    timer !== null ? clearTimeout(timer) : null;
    resolve();
  });
}

function getFuncWithArgsOne(func, timeout, isStressTest) {
  return new Promise(async (resolve, reject) => {
    let timer = null;
    if (!isStressTest) {
      timer = setTimeout(() => {
        reject(new Error("execute timeout " + timeout + "ms"));
      }, timeout);
    }

    function done() {
      timer !== null ? clearTimeout(timer) : null;
      resolve();
    }

    try {
      await func(done);
    } catch (err) {
      timer !== null ? clearTimeout(timer) : null;
      reject(err);
    }
  });
}

function getFuncWithArgsTwo(func, timeout, paramItem, isStressTest) {
  return new Promise(async (resolve, reject) => {
    let timer = null;
    if (!isStressTest) {
      timer = setTimeout(() => {
        reject(new Error("execute timeout " + timeout + "ms"));
      }, timeout);
    }

    function done() {
      timer !== null ? clearTimeout(timer) : null;
      resolve();
    }

    try {
      await func(done, paramItem);
    } catch (err) {
      timer !== null ? clearTimeout(timer) : null;
      reject(err);
    }
  });
}

function processFunc(coreContext, func) {
  let argNames = ((func || "")
    .toString()
    .replace(/((\/\/.*$)|(\/\*[\s\S]*?\*\/))/gm, "")
    .match(/^(function)?\s*[^\(]*\(\s*([^\)]*)\)/m) || ["", "", ""])[2]
    .split(",") // split parameters
    .map((item) =>
      item.replace(/^\s*(_?)(.+?)\1\s*$/, (name) => name.split("=")[0].trim())
    )
    .filter(String);
  let funcLen = func.length;
  let processedFunc;
  const config = coreContext.getDefaultService("config");
  config.setSupportAsync(true);
  const timeout = +(config.timeout === undefined ? 5000 : config.timeout);
  const isStressTest =
    coreContext.getServices("dataDriver") !== undefined ||
    config.getStress() > 1;
  switch (funcLen) {
    case 0: {
      processedFunc = function () {
        return getFuncWithArgsZero(func, timeout, isStressTest);
      };
      break;
    }
    case 1: {
      processedFunc = function () {
        return getFuncWithArgsOne(func, timeout, isStressTest);
      };
      break;
    }
    default: {
      processedFunc = function (paramItem) {
        return getFuncWithArgsTwo(func, timeout, paramItem, isStressTest);
      };
      break;
    }
  }
  return processedFunc;
}

function secureRandomNumber() {
  return crypto.randomBytes(8).readUInt32LE() / 0xffffffff;
}

class SuiteService {
  constructor(attr) {
    this.id = attr.id;
    this.rootSuite = new SuiteService.Suite({});
    this.currentRunningSuite = this.rootSuite;
    this.suitesStack = [this.rootSuite];
    this.targetSuiteArray = [];
    this.targetSpecArray = [];
    this.currentRunningSuiteDesc = null;
    this.fullRun = false;
    this.isSkipSuite = false;
    this.suiteSkipReason = null;
  }

  describe(desc, func) {
    const configService = this.coreContext.getDefaultService("config");
    if (
      this.suitesStack.some((suite) => {
        return suite.description === desc;
      })
    ) {
      console.error(`${TAG} Loop nesting occurs : ${desc}`);
      this.suiteSkipReason = "";
      this.isSkipSuite = false;
      return;
    }
    let isFilter = this.analyzeConfigServiceClass(configService.class, desc);
    if (configService.filterSuite(desc) && isFilter) {
      if (
        this.currentRunningSuite.description === "" ||
        this.currentRunningSuite.description == null
      ) {
        console.info(`${TAG}filter suite : ${desc}`);
        this.suiteSkipReason = "";
        this.isSkipSuite = false;
        return;
      }
    }
    const suite = new SuiteService.Suite({ description: desc });
    if (this.isSkipSuite) {
      suite.isSkip = true;
      suite.skipReason = this.suiteSkipReason;
    }
    this.suiteSkipReason = "";
    this.isSkipSuite = false;
    if (
      typeof this.coreContext.getServices("dataDriver") !== "undefined" &&
      configService["dryRun"] !== "true"
    ) {
      let suiteStress = this.coreContext
        .getServices("dataDriver")
        .dataDriver.getSuiteStress(desc);
      for (let i = 1; i < suiteStress; i++) {
        this.currentRunningSuite.childSuites.push(suite);
      }
    }
    this.currentRunningSuite.childSuites.push(suite);
    this.currentRunningSuite = suite;
    this.suitesStack.push(suite);
    func.call();
    this.suitesStack.pop();
    this.currentRunningSuite = this.suitesStack.pop();
    this.suitesStack.push(this.currentRunningSuite);
  }
  xdescribe(desc, func, reason) {
    const configService = this.coreContext.getDefaultService("config");
    if (!configService.skipMessage && configService.runSkipped !== "all") {
      if (configService.runSkipped != null && configService.runSkipped !== "") {
        let finalDesc = "";
        this.suitesStack.map((suite) => {
          finalDesc = finalDesc + "." + suite.description;
        });
        finalDesc = (finalDesc + "." + desc).substring(2);
        console.info(`${TAG} finalDesc ${finalDesc}`);
        if (configService.checkIfSuiteInSkipRun(finalDesc)) {
          console.info(`${TAG} runSkipped suite: ${desc}`);
        } else {
          console.info(
            reason == null
              ? `${TAG} skip suite: ${desc}`
              : `${TAG} skip suite: ${desc}, and the reason is ${reason}`
          );
          return;
        }
      } else {
        console.info(
          reason == null
            ? `${TAG} skip suite: ${desc}`
            : `${TAG} skip suite: ${desc}, and the reason is ${reason}`
        );
        return;
      }
    }
    this.isSkipSuite = true;
    this.suiteSkipReason = reason;
    this.describe(desc, func);
  }

  beforeAll(func) {
    this.currentRunningSuite.beforeAll.push(
      processFunc(this.coreContext, func)
    );
  }

  beforeEach(func) {
    this.currentRunningSuite.beforeEach.push(
      processFunc(this.coreContext, func)
    );
  }

  beforeItSpecified(itDescs, func) {
    this.currentRunningSuite.beforeItSpecified.set(
      itDescs,
      processFunc(this.coreContext, func)
    );
  }

  afterItSpecified(itDescs, func) {
    this.currentRunningSuite.afterItSpecified.set(
      itDescs,
      processFunc(this.coreContext, func)
    );
  }

  afterAll(func) {
    this.currentRunningSuite.afterAll.push(processFunc(this.coreContext, func));
  }

  afterEach(func) {
    this.currentRunningSuite.afterEach.push(
      processFunc(this.coreContext, func)
    );
  }

  getCurrentRunningSuite() {
    return this.currentRunningSuite;
  }

  setCurrentRunningSuite(suite) {
    this.currentRunningSuite = suite;
  }

  getRootSuite() {
    return this.rootSuite;
  }

  getCurrentRunningSuiteDesc() {
    return this.currentRunningSuiteDesc;
  }

  setCurrentRunningSuiteDesc(suite, currentSuite, prefix) {
    if (suite != null && suite === currentSuite) {
      this.currentRunningSuiteDesc = prefix;
    } else if (suite != null && suite !== currentSuite) {
      suite.childSuites.forEach((it) => {
        let temp = prefix;
        if (it.description != null || it.description !== "") {
          temp = prefix === "" ? it.description : prefix + "." + it.description;
        }
        this.setCurrentRunningSuiteDesc(it, currentSuite, temp);
      });
    }
  }
  analyzeConfigServiceClass(configServiceClass, desc) {
    if (configServiceClass == null || configServiceClass === "") {
      this.fullRun = true;
      return false;
    }
    if (this.targetSuiteArray.length === 0) {
      const targetArray = configServiceClass.split(",");
      for (let index in targetArray) {
        if (targetArray[index].includes("#")) {
          this.targetSpecArray.push(targetArray[index]);
        } else {
          this.targetSuiteArray.push(targetArray[index]);
        }
      }
    }
    return !configServiceClass.includes(desc);
  }
  traversalResults(suite, obj, breakOnError) {
    if (suite.childSuites.length === 0 && suite.specs.length === 0) {
      return;
    }
    if (suite.specs.length > 0) {
      for (const itItem of suite.specs) {
        obj.total++;
        let itInfo = {
          currentThreadName: "mainThread",
          description: suite.description + "#" + itItem.description,
          result: -3
        };
        if (SysTestKit.workerPort !== null) {
          itInfo.currentThreadName = SysTestKit.workerPort.name;
        }
        obj.itItemList.push(itInfo);
        if (breakOnError && (obj.error > 0 || obj.failure > 0)) {
          // breakOnError模式
          continue;
        }
        if (itItem.error) {
          obj.error++;
          itInfo.result = -1;
        } else if (itItem.fail) {
          obj.failure++;
          itInfo.result = -2;
        } else if (itItem.pass === true) {
          obj.pass++;
          itInfo.result = 0;
        }
      }
    }

    obj.duration += suite.duration;

    if (suite.childSuites.length > 0) {
      for (const suiteItem of suite.childSuites) {
        this.traversalResults(suiteItem, obj, breakOnError);
      }
    }
  }

  async setSuiteResults(suite, error, coreContext) {
    if (suite.childSuites.length === 0 && suite.specs.length === 0) {
      return;
    }
    if (suite.specs.length > 0) {
      const specService = coreContext.getDefaultService("spec");
      for (const specItem of suite.specs) {
        specService.setCurrentRunningSpec(specItem);
        if (error instanceof AssertException) {
          specItem.fail = error;
        } else {
          specItem.error = error;
        }
        await coreContext.fireEvents("spec", "specStart", specItem);
        await coreContext.fireEvents("spec", "specDone", specItem);
      }
    }
    if (suite.childSuites.length > 0) {
      for (const suiteItem of suite.childSuites) {
        await this.setSuiteResults(suiteItem, error, coreContext);
      }
    }
  }

  getSummary() {
    let suiteService = this.coreContext.getDefaultService("suite");
    let rootSuite = suiteService.rootSuite;
    const specService = this.coreContext.getDefaultService("spec");
    const configService = this.coreContext.getDefaultService("config");
    let breakOnError = configService.isBreakOnError();
    let isError = specService.getStatus();
    let isBreaKOnError = breakOnError && isError;
    // itItemList 保存当前用例执行情况, 发送到主线程用例计算最终结果
    let obj = {
      total: 0,
      failure: 0,
      error: 0,
      pass: 0,
      ignore: 0,
      duration: 0,
      itItemList: []
    };
    for (const suiteItem of rootSuite.childSuites) {
      this.traversalResults(suiteItem, obj, isBreaKOnError);
    }
    obj.ignore = obj.total - obj.pass - obj.failure - obj.error;
    return obj;
  }

  init(coreContext) {
    this.coreContext = coreContext;
  }

  traversalSuites(suite, obj, configService) {
    if (suite.childSuites.length === 0 && suite.specs.length === 0) {
      return [];
    }
    if (suite.specs.length > 0) {
      let itArray = [];
      for (const itItem of suite["specs"]) {
        if (
          !configService.filterDesc(
            suite.description,
            itItem.description,
            itItem.fi,
            null
          )
        ) {
          itArray.push({ itName: itItem.description });
        }
      }
      obj[suite.description] = itArray;
    }
    if (suite.childSuites.length > 0) {
      let suiteArray = [];
      for (const suiteItem of suite.childSuites) {
        let suiteObj = {};
        this.traversalSuites(suiteItem, suiteObj, configService);
        if (!configService.filterSuite(suiteItem.description)) {
          suiteArray.push(suiteObj);
        }
      }
      obj.suites = suiteArray;
    }
  }

  async dryRun(abilityDelegator) {
    console.info(`${TAG} rootSuite : ` + JSON.stringify(this.rootSuite));
    let obj = this.rootSuite;
    let prefixStack = [];
    let suiteArray = [];
    let skipSuiteArray = [];
    this.analyzeSuitesArray(prefixStack, suiteArray, skipSuiteArray, obj);
    const configService = this.coreContext.getDefaultService("config");
    let result;
    if (configService.skipMessage) {
      result = { suites: suiteArray, skipSuites: skipSuiteArray };
    } else {
      result = { suites: suiteArray };
    }
    let strJson = JSON.stringify(result);
    let strLen = strJson.length;
    let maxLen = 500;
    let maxCount = Math.floor(strLen / maxLen);
    for (let count = 0; count <= maxCount; count++) {
      await SysTestKit.print(
        strJson.substring(count * maxLen, (count + 1) * maxLen)
      );
    }
    console.info(`${TAG}dryRun print success`);
    abilityDelegator.finishTest("dry run finished!!!", 0, () => {});
  }

  //将suitesArray的嵌套结构展开成三层结构
  analyzeSuitesArray(prefixStack, suiteArray, skipSuiteArray, obj) {
    obj.childSuites.map((suite) => {
      if (suite.description != null && suite.description !== "") {
        let prefix = "";
        if (prefixStack.length > 0) {
          prefix = prefixStack.join(".") + "." + suite.description;
        } else {
          prefix = suite.description;
        }
        prefixStack.push(suite.description);
        let temp = {};
        temp[prefix] = [];
        let skipTemp = {};
        skipTemp[prefix] = [];
        suite.specs.map((spec) => {
          let it = { itName: spec.description };
          spec.isSkip ? skipTemp[prefix].push(it) : temp[prefix].push(it);
        });
        suiteArray.push(temp);
        skipSuiteArray.push(skipTemp);
      }
      this.analyzeSuitesArray(prefixStack, suiteArray, skipSuiteArray, suite);
      prefixStack.pop();
    });
  }
  //获取当前测试套下的所有测试用例数量
  getAllChildSuiteNum(suite, specArray) {
    if (suite.specs != null) {
      suite.specs.forEach((spec) => specArray.push(spec));
    }
    if (suite.childSuites != null) {
      suite.childSuites.forEach((it) =>
        this.getAllChildSuiteNum(it, specArray)
      );
    }
  }

  execute() {
    const configService = this.coreContext.getDefaultService("config");
    if (configService.filterValid.length !== 0) {
      this.coreContext.fireEvents("task", "incorrectFormat");
      return;
    }
    if (configService.filterXdescribe.length !== 0) {
      this.coreContext.fireEvents("task", "incorrectTestSuiteFormat");
      return;
    }
    if (configService.isRandom() && this.rootSuite.childSuites.length > 0) {
      this.rootSuite.childSuites.sort(function () {
        return +("0." + (+new Date() + "").split("").reverse().join("")) > 0.5
          ? -1
          : 1;
      });
      this.currentRunningSuite = this.rootSuite.childSuites[0];
    }
    if (configService.isSupportAsync()) {
      console.info(`${TAG} rootSuite:` + JSON.stringify(this.rootSuite));
      let asyncExecute = async () => {
        await this.coreContext.fireEvents("task", "taskStart");
        await this.rootSuite.asyncRun(this.coreContext);
      };
      asyncExecute().then(async () => {
        await this.coreContext.fireEvents("task", "taskDone");
      });
    } else {
      console.info("${TAG} rootSuite:" + JSON.stringify(this.rootSuite));
      this.coreContext.fireEvents("task", "taskStart");
      this.rootSuite.run(this.coreContext);
      this.coreContext.fireEvents("task", "taskDone");
    }
  }

  apis() {
    const _this = this;
    return {
      describe: function (desc, func) {
        return _this.describe(desc, func);
      },
      xdescribe: function (desc, func, reason) {
        return _this.xdescribe(desc, func, reason);
      },
      beforeItSpecified: function (itDescs, func) {
        return _this.beforeItSpecified(itDescs, func);
      },
      afterItSpecified: function (itDescs, func) {
        return _this.afterItSpecified(itDescs, func);
      },
      beforeAll: function (func) {
        return _this.beforeAll(func);
      },
      beforeEach: function (func) {
        return _this.beforeEach(func);
      },
      afterAll: function (func) {
        return _this.afterAll(func);
      },
      afterEach: function (func) {
        return _this.afterEach(func);
      }
    };
  }
}

SuiteService.Suite = class {
  constructor(attrs) {
    this.description = attrs.description || "";
    this.childSuites = [];
    this.specs = [];
    this.beforeAll = [];
    this.afterAll = [];
    this.beforeItSpecified = new Map();
    this.afterItSpecified = new Map();
    this.beforeEach = [];
    this.afterEach = [];
    this.duration = 0;
    this.hookError = null;
    this.isSkip = false;
    this.skipReason = "";
  }

  pushSpec(spec) {
    this.specs.push(spec);
  }

  removeSpec(desc) {
    this.specs = this.specs.filter((item, index) => {
      return item.description !== desc;
    });
  }

  getSpecsNum() {
    return this.specs.length;
  }

  isRun(coreContext) {
    const configService = coreContext.getDefaultService("config");
    const suiteService = coreContext.getDefaultService("suite");
    const specService = coreContext.getDefaultService("spec");
    let breakOnError = configService.isBreakOnError();
    let isError = specService.getStatus();
    return breakOnError && isError;
  }

  run(coreContext) {
    const suiteService = coreContext.getDefaultService("suite");
    suiteService.setCurrentRunningSuite(this);
    if (this.description !== "") {
      coreContext.fireEvents("suite", "suiteStart", this);
    }
    this.runHookFunc("beforeAll");
    if (this.specs.length > 0) {
      const configService = coreContext.getDefaultService("config");
      if (configService.isRandom()) {
        this.specs.sort(function () {
          return +("0." + (+new Date() + "").split("").reverse().join("")) > 0.5
            ? -1
            : 1;
        });
      }
      for (let spec in this.specs) {
        let isBreakOnError = this.isRun(coreContext);
        if (isBreakOnError) {
          break;
        }
        this.runHookFunc("beforeEach");
        spec.run(coreContext);
        this.runHookFunc("afterEach");
      }
    }
    if (this.childSuites.length > 0) {
      for (let suite in this.childSuites) {
        let isBreakOnError = this.isRun(coreContext);
        if (isBreakOnError) {
          break;
        }
        suite.run(coreContext);
        suiteService.setCurrentRunningSuite(suite);
      }
    }
    this.runHookFunc("afterAll");
    if (this.description !== "") {
      coreContext.fireEvents("suite", "suiteDone");
    }
  }

  async runBeforeItSpecified(beforeItSpecified, specItem) {
    for (const [itNames, hookFunc] of beforeItSpecified) {
      if (
        (Object.prototype.toString.call(itNames) === "[object Array]" &&
          itNames.includes(specItem.description)) ||
        (Object.prototype.toString.call(itNames) === "[object String]" &&
          itNames === specItem.description)
      ) {
        await Reflect.apply(hookFunc, null, []);
      }
      break;
    }
  }

  async runAfterItSpecified(beforeItSpecified, specItem) {
    for (const [itNames, hookFunc] of beforeItSpecified) {
      if (
        (Object.prototype.toString.call(itNames) === "[object Array]" &&
          itNames.includes(specItem.description)) ||
        (Object.prototype.toString.call(itNames) === "[object String]" &&
          itNames === specItem.description)
      ) {
        await Reflect.apply(hookFunc, null, []);
      }
      break;
    }
  }

  async asyncRunSpecs(coreContext) {
    const configService = coreContext.getDefaultService("config");
    if (configService.isRandom()) {
      this.specs.sort(function () {
        return +("0." + (+new Date() + "").split("").reverse().join("")) > 0.5
          ? -1
          : 1;
      });
    }
    const specService = coreContext.getDefaultService("spec");
    for (let specItem of this.specs) {
      specService.setCurrentRunningSpec(specItem);
      // 遇错即停模式,发现用例有问题，直接返回，不在执行后面的it
      let isBreakOnError = this.isRun(coreContext);
      if (isBreakOnError) {
        console.info("break description :" + this.description);
        break;
      }
      await coreContext.fireEvents("spec", "specStart", specItem);
      try {
        await this.runBeforeItSpecified(this.beforeItSpecified, specItem);
        await this.runAsyncHookFunc("beforeEach");
        await specItem.asyncRun(coreContext);
        await this.runAfterItSpecified(this.afterItSpecified, specItem);
        await this.runAsyncHookFunc("afterEach");
      } catch (e) {
        console.error(`${TAG}stack:${e?.stack}`);
        console.error(`${TAG}stack end`);
        if (e instanceof AssertException) {
          specItem.fail = e;
        } else {
          specItem.error = e;
        }
        specService.setStatus(true);
      }
      specItem.setResult();
      await coreContext.fireEvents("spec", "specDone", specItem);
      specService.setCurrentRunningSpec(null);
    }
  }

  async asyncRunChildSuites(coreContext) {
    for (let i = 0; i < this.childSuites.length; i++) {
      // 遇错即停模式, 发现用例有问题，直接返回，不在执行后面的description
      let isBreakOnError = this.isRun(coreContext);
      if (isBreakOnError) {
        console.info(`${TAG}break description : ${this.description}`);
        break;
      }
      await this.childSuites[i].asyncRun(coreContext);
    }
  }

  async asyncRun(coreContext) {
    const suiteService = coreContext.getDefaultService("suite");
    const specService = coreContext.getDefaultService("spec");

    suiteService.setCurrentRunningSuite(this);
    suiteService.suitesStack.push(this);
    if (this.description !== "") {
      await coreContext.fireEvents("suite", "suiteStart", this);
    }

    try {
      await this.runAsyncHookFunc("beforeAll");
    } catch (error) {
      console.error(`${TAG}${error?.stack}`);
      this.hookError = error;
    }

    if (this.hookError !== null) {
      specService.setStatus(true);
      await suiteService.setSuiteResults(this, this.hookError, coreContext);
    }

    if (this.specs.length > 0 && this.hookError === null) {
      await this.asyncRunSpecs(coreContext);
    }

    if (this.childSuites.length > 0 && this.hookError === null) {
      await this.asyncRunChildSuites(coreContext);
    }

    try {
      await this.runAsyncHookFunc("afterAll");
    } catch (error) {
      console.error(`${TAG}${error?.stack}`);
      this.hookError = error;
      specService.setStatus(true);
    }

    if (this.description !== "") {
      await coreContext.fireEvents("suite", "suiteDone");
      let childSuite = suiteService.suitesStack.pop();
      let currentRunningSuite = suiteService.suitesStack.pop();
      suiteService.setCurrentRunningSuite(currentRunningSuite);
      suiteService.suitesStack.push(currentRunningSuite);
    }
  }

  runHookFunc(hookName) {
    if (this[hookName] && this[hookName].length > 0) {
      this[hookName].forEach((func) => {
        try {
          func();
        } catch (e) {
          console.error(`${TAG}${e.stack}`);
        }
      });
    }
  }

  async runAsyncHookFunc(hookName) {
    for (const hookItem of this[hookName]) {
      try {
        await hookItem();
      } catch (error) {
        error["message"] += `, error in ${hookName} function`;
        throw error;
      }
    }
  }
};

class SpecService {
  constructor(attr) {
    this.id = attr.id;
    this.totalTest = 0;
    this.hasError = false;
    this.skipSpecNum = 0;
    this.isSkipSpec = false;
    this.specSkipReason = "";
  }

  init(coreContext) {
    this.coreContext = coreContext;
  }

  setCurrentRunningSpec(spec) {
    this.currentRunningSpec = spec;
  }

  setStatus(obj) {
    this.hasError = obj;
  }

  getStatus() {
    return this.hasError;
  }

  getTestTotal() {
    return this.totalTest;
  }

  getCurrentRunningSpec() {
    return this.currentRunningSpec;
  }

  getSkipSpecNum() {
    return this.skipSpecNum;
  }

  initSpecService() {
    this.isSkipSpec = false;
    this.specSkipReason = "";
  }

  it(desc, filter, func) {
    const suiteService = this.coreContext.getDefaultService("suite");
    const configService = this.coreContext.getDefaultService("config");
    let isFilter = new NestFilter().filterNestName(
      suiteService.targetSuiteArray,
      suiteService.targetSpecArray,
      suiteService.suitesStack,
      desc
    );
    if (configService.filterWithNest(desc, filter)) {
      console.info(`${TAG}filter it :${desc}`);
      this.initSpecService();
      return;
    }
    if (
      configService.filterDesc(
        suiteService.currentRunningSuite.description,
        desc,
        filter,
        this.coreContext
      ) &&
      isFilter &&
      !suiteService.fullRun
    ) {
      console.info(`${TAG}filter it :${desc}`);
      this.initSpecService();
    } else {
      let processedFunc = processFunc(this.coreContext, func);
      const spec = new SpecService.Spec({
        description: desc,
        fi: filter,
        fn: processedFunc
      });
      if (this.isSkipSpec) {
        spec.isSkip = true;
        spec.skipReason = this.specSkipReason;
      }
      this.initSpecService();
      if (configService.runSkipped === "skipped" && !spec.isSkip) {
        console.info(
          `${TAG} runSkipped is skipped , just run xit, don't run it: ${spec.description}`
        );
        return;
      }
      if (suiteService.getCurrentRunningSuite().isSkip && !spec.isSkip) {
        configService.filterXdescribe.push(
          suiteService.getCurrentRunningSuite().description
        );
      }
      if (
        typeof this.coreContext.getServices("dataDriver") !== "undefined" &&
        configService["dryRun"] !== "true"
      ) {
        let specStress = this.coreContext
          .getServices("dataDriver")
          .dataDriver.getSpecStress(desc);
        for (let i = 1; i < specStress; i++) {
          this.totalTest++;
          suiteService.getCurrentRunningSuite().pushSpec(spec);
        }
      }
      // dryRun 状态下不统计压力测试重复数据
      if (configService["dryRun"] !== "true") {
        let stress = configService.getStress(); // 命令配置压力测试
        console.info(`${TAG}stress length : ${stress}`);
        for (let i = 1; i < stress; i++) {
          this.totalTest++;
          suiteService.getCurrentRunningSuite().pushSpec(spec);
        }
      }
      this.totalTest++;
      suiteService.getCurrentRunningSuite().pushSpec(spec);
    }
  }

  xit(desc, filter, func, reason) {
    const configService = this.coreContext.getDefaultService("config");
    const suiteService = this.coreContext.getDefaultService("suite");
    if (!configService.skipMessage && configService.runSkipped !== "all") {
      if (configService.runSkipped != null && configService.runSkipped !== "") {
        let finalDesc = "";
        suiteService.suitesStack.map((suite) => {
          finalDesc = finalDesc + "." + suite.description;
        });
        finalDesc = (finalDesc + "#" + desc).substring(2);
        if (configService.checkIfSpecInSkipRun(finalDesc)) {
          console.info(`${TAG} runSkipped spec: ${desc}`);
        } else {
          console.info(
            reason == null
              ? `${TAG} skip spec: ${desc}`
              : `${TAG} skip spec: ${desc}, and the reason is ${reason}`
          );
          return;
        }
      } else {
        console.info(
          reason == null
            ? `${TAG} skip spec: ${desc}`
            : `${TAG} skip spec: ${desc}, and the reason is ${reason}`
        );
        return;
      }
    }
    this.skipSpecNum++;
    this.isSkipSpec = true;
    this.specSkipReason = reason;
    this.it(desc, filter, func);
  }

  apis() {
    const _this = this;
    return {
      it: function (desc, filter, func) {
        return _this.it(desc, filter, func);
      },
      xit: function (desc, filter, func, reason) {
        return _this.xit(desc, filter, func, reason);
      }
    };
  }
}

SpecService.Spec = class {
  constructor(attrs) {
    this.description = attrs.description || "";
    this.fi = attrs.fi;
    this.fn = attrs.fn || function () {};
    this.fail = undefined;
    this.error = undefined;
    this.duration = 0;
    this.startTime = 0;
    this.isExecuted = false; // 当前用例是否执行
    this.isSkip = false;
    this.skipReason = "";
    this.expectMsg = "";
  }

  setResult() {
    if (this.fail) {
      this.pass = false;
    } else {
      this.pass = true;
    }
  }

  run(coreContext) {
    const specService = coreContext.getDefaultService("spec");
    specService.setCurrentRunningSpec(this);
    coreContext.fireEvents("spec", "specStart", this);
    this.isExecuted = true;
    try {
      let dataDriver = coreContext.getServices("dataDriver");
      if (typeof dataDriver === "undefined") {
        this.fn();
      } else {
        let suiteParams = dataDriver.dataDriver.getSuiteParams();
        let specParams = dataDriver.dataDriver.getSpecParams();
        console.info(`${TAG}[suite params] ${JSON.stringify(suiteParams)}`);
        console.info(`${TAG}[spec params] ${JSON.stringify(specParams)}`);
        if (this.fn.length === 0) {
          this.fn();
        } else if (specParams.length === 0) {
          this.fn(suiteParams);
        } else {
          specParams.forEach((paramItem) =>
            this.fn(Object.assign({}, paramItem, suiteParams))
          );
        }
      }
      this.setResult();
    } catch (e) {
      this.error = e;
      specService.setStatus(true);
    }
    coreContext.fireEvents("spec", "specDone", this);
  }

  async asyncRun(coreContext) {
    const dataDriver = coreContext.getServices("dataDriver");
    if (typeof dataDriver === "undefined") {
      await this.fn();
    } else {
      const suiteParams = dataDriver.dataDriver.getSuiteParams();
      const specParams = dataDriver.dataDriver.getSpecParams();
      console.info(`[suite params] ${JSON.stringify(suiteParams)}`);
      console.info(`[spec params] ${JSON.stringify(specParams)}`);
      if (this.fn.length === 0) {
        await this.fn();
      } else if (specParams.length === 0) {
        await this.fn(suiteParams);
      } else {
        for (const paramItem of specParams) {
          await this.fn(Object.assign({}, paramItem, suiteParams));
        }
      }
    }

    this.isExecuted = true;
  }

  filterCheck(coreContext) {
    const specService = coreContext.getDefaultService("spec");
    specService.setCurrentRunningSpec(this);
    return true;
  }
};

class ExpectService {
  constructor(attr) {
    this.id = attr.id;
    this.matchers = {};
    this.customMatchers = [];
  }

  expect(actualValue) {
    return this.wrapMatchers(actualValue);
  }

  init(coreContext) {
    this.coreContext = coreContext;
    this.addMatchers(this.basicMatchers());
  }

  addMatchers(matchers) {
    for (const matcherName in matchers) {
      if (Object.prototype.hasOwnProperty.call(matchers, matcherName)) {
        this.matchers[matcherName] = matchers[matcherName];
      }
    }
  }

  removeMatchers(customAssertionName) {
    if (customAssertionName === "all") {
      for (const matcherName in this.matchers) {
        this.matchers[matcherName] = this.customMatchers.includes(matcherName)
          ? () => {
              throw new Error(`${matcherName} is unregistered`);
            }
          : undefined;
      }
    } else {
      this.matchers[customAssertionName] = () => {
        throw new Error(`${customAssertionName} is unregistered`);
      };
    }
  }

  basicMatchers() {
    return {
      assertTrue: assertTrueFun,
      assertEqual: assertEqualFun,
      assertThrow: assertThrowFun
    };
  }

  initWrapMatchers(currentRunningSpec) {
    return {
      // 翻转标识
      isNot: false,
      // 翻转方法
      not: function () {
        this.isNot = true;
        return this;
      },
      message: function (msg) {
        currentRunningSpec.expectMsg = msg;
        console.info(`${TAG} msg: ${msg}`);
        return this;
      }
    };
  }

  handleWithAssertPromise(
    _this,
    wrappedMatchers,
    matcherName,
    actualValue,
    currentRunningSpec,
    currentRunningSuite
  ) {
    wrappedMatchers[matcherName] = async function (...args) {
      await _this.matchers[matcherName](actualValue, args).then(
        function (result) {
          if (wrappedMatchers.isNot) {
            result.pass = !result.pass;
          }
          result.actualValue = actualValue;
          result.checkFunc = matcherName;
          if (!result.pass) {
            const assertError = new AssertException(result.message);
            currentRunningSpec
              ? (currentRunningSpec.fail = assertError)
              : (currentRunningSuite.hookError = assertError);
            throw assertError;
          }
        }
      );
    };
  }

  handleWithoutAssertPromise(
    _this,
    wrappedMatchers,
    matcherName,
    actualValue,
    currentRunningSpec,
    currentRunningSuite
  ) {
    wrappedMatchers[matcherName] = function (...args) {
      const result = _this.customMatchers.includes(matcherName)
        ? _this.matchers[matcherName](actualValue, args[0])
        : _this.matchers[matcherName](actualValue, args);
      if (wrappedMatchers.isNot) {
        result.pass = !result.pass;
        result.message = LogExpectError.getErrorMsg(
          matcherName,
          actualValue,
          args[0],
          result.message
        );
      }
      result.actualValue = actualValue;
      result.checkFunc = matcherName;
      if (!result.pass) {
        const assertError = new AssertException(result.message);
        currentRunningSpec
          ? (currentRunningSpec.fail = assertError)
          : (currentRunningSuite.hookError = assertError);
        throw assertError;
      }
    };
  }

  addAssert(wrappedMatchers, matcherName, actualValue) {
    const _this = this;
    const specService = _this.coreContext.getDefaultService("spec");
    const currentRunningSpec = specService.getCurrentRunningSpec();
    const currentRunningSuite = _this.coreContext
      .getDefaultService("suite")
      .getCurrentRunningSuite();
    if (matcherName.search("assertPromise") === 0) {
      this.handleWithAssertPromise(
        _this,
        wrappedMatchers,
        matcherName,
        actualValue,
        currentRunningSpec,
        currentRunningSuite
      );
    } else {
      this.handleWithoutAssertPromise(
        _this,
        wrappedMatchers,
        matcherName,
        actualValue,
        currentRunningSpec,
        currentRunningSuite
      );
    }
  }

  wrapMatchers(actualValue) {
    const _this = this;
    const specService = _this.coreContext.getDefaultService("spec");
    const currentRunningSpec = specService.getCurrentRunningSpec();
    const wrappedMatchers = this.initWrapMatchers(currentRunningSpec);
    const currentRunningSuite = _this.coreContext
      .getDefaultService("suite")
      .getCurrentRunningSuite();
    for (const matcherName in this.matchers) {
      let result = Object.prototype.hasOwnProperty.call(
        this.matchers,
        matcherName
      );
      if (!result) {
        continue;
      }
      this.addAssert(wrappedMatchers, matcherName, actualValue);
    }
    return wrappedMatchers;
  }

  apis() {
    const _this = this;
    return {
      expect: function (actualValue) {
        return _this.expect(actualValue);
      }
    };
  }
}

class ReportService {
  constructor(attr) {
    this.id = attr.id;
  }

  init(coreContext) {
    this.coreContext = coreContext;
    this.specService = this.coreContext.getDefaultService("spec");
    this.suiteService = this.coreContext.getDefaultService("suite");
    this.duration = 0;
  }

  taskStart() {
    console.info(`${TAG}[start] start run suites`);
  }

  async suiteStart() {
    console.info(
      `${TAG}[suite start]${this.suiteService.getCurrentRunningSuite().description}`
    );
  }

  async specStart() {
    console.info(
      `${TAG}start running case '${this.specService.currentRunningSpec.description}'`
    );
    this.index = this.index + 1;
    let spec = this.specService.currentRunningSpec;
    spec.startTime = await SysTestKit.getRealTime();
  }

  async specDone() {
    let msg = "";
    let spec = this.specService.currentRunningSpec;
    let suite = this.suiteService.currentRunningSuite;
    spec.duration = (await SysTestKit.getRealTime()) - spec.startTime;
    suite.duration += spec.duration;
    if (spec.error) {
      this.formatPrint(
        "error",
        spec.description + " ; consuming " + spec.duration + "ms"
      );
      this.formatPrint("errorDetail", spec.error);
    } else if (spec.fail) {
      this.formatPrint(
        "fail",
        spec.description + " ; consuming " + spec.duration + "ms"
      );
      this.formatPrint("failDetail", spec.fail?.message);
    } else {
      this.formatPrint(
        "pass",
        spec.description + " ; consuming " + spec.duration + "ms"
      );
    }
    this.formatPrint(this.specService.currentRunningSpec.error, msg);
  }

  suiteDone() {
    let suite = this.suiteService.currentRunningSuite;
    let message = suite.hookError ? `, ${suite.hookError?.message}` : "";
    console.info(
      `[suite end] ${suite.description} consuming ${suite.duration} ms${message}`
    );
  }

  taskDone() {
    let msg = "";
    let summary = this.suiteService.getSummary();
    msg =
      "total cases:" +
      summary.total +
      ";failure " +
      summary.failure +
      "," +
      "error " +
      summary.error;
    msg += ",pass " + summary.pass + "; consuming " + summary.duration + "ms";
    console.info(`${TAG}${msg}`);
    console.info(`${TAG}[end] run suites end`);
  }

  incorrectFormat() {
    if (this.coreContext.getDefaultService("config").filterValid.length !== 0) {
      this.coreContext
        .getDefaultService("config")
        .filterValid.forEach(function (item) {
          console.info(`${TAG}this param ${item} is invalid`);
        });
    }
  }

  incorrectTestSuiteFormat() {
    if (
      this.coreContext.getDefaultService("config").filterXdescribe.length !== 0
    ) {
      this.coreContext
        .getDefaultService("config")
        .filterXdescribe.forEach(function (item) {
          console.info(`${TAG}xdescribe: ${item} should not contain it`);
        });
    }
  }

  formatPrint(type, msg) {
    switch (type) {
      case "pass":
        console.info(`${TAG}[pass]${msg}`);
        break;
      case "fail":
        console.info(`${TAG}[fail]${msg}`);
        break;
      case "failDetail":
        console.info(`${TAG}[failDetail]${msg}`);
        break;
      case "error":
        console.info(`${TAG}[error]${msg}`);
        break;
      case "errorDetail":
        console.info(`${TAG}[errorDetail]${msg}`);
        break;
    }
  }

  sleep(numberMillis) {
    var now = new Date();
    var exitTime = now.getTime() + numberMillis;
    while (true) {
      now = new Date();
      if (now.getTime() > exitTime) {
        return;
      }
    }
  }
}

export { SuiteService, SpecService, ExpectService, ReportService };
