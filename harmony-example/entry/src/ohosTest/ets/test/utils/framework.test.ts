import Core from "./core/core";
import { DEFAULT, TestType, Size, Level, TAG, PrintTag } from "./core/Constant";
import DataDriver from "./core/module/config/DataDriver";
import ExpectExtend from "./core/module/assert/ExpectExtend";
import OhReport from "./core/module/report/OhReport";
import SysTestKit from "./core/module/kit/SysTestKit";
import {
  describe,
  beforeAll,
  beforeEach,
  afterEach,
  afterAll,
  it,
  expect,
  beforeItSpecified,
  afterItSpecified,
  xdescribe,
  xit
} from "./core/interface";
import { MockKit, when } from "./core/module/mock/MockKit";
import ArgumentMatchers from "./core/module/mock/ArgumentMatchers";
import worker from "@ohos.worker";

class Hypium {
  static context = new Map();
  static setData(data) {
    const core = Core.getInstance();
    const dataDriver = new DataDriver({ data });
    core.addService("dataDriver", dataDriver);
  }

  static setTimeConfig(systemTime) {
    SysTestKit.systemTime = systemTime;
  }

  static set(key, value) {
    Hypium.context.set(key, value);
  }

  static get(key) {
    return Hypium.context.get(key);
  }

  static hypiumTest(abilityDelegator, abilityDelegatorArguments, testsuite) {
    const core = Core.getInstance();
    const expectExtend = new ExpectExtend({
      id: "extend"
    });
    core.addService("expect", expectExtend);
    const ohReport = new OhReport({
      delegator: abilityDelegator,
      abilityDelegatorArguments: abilityDelegatorArguments
    });
    SysTestKit.delegator = abilityDelegator;
    core.addService("report", ohReport);
    core.init();
    core.subscribeEvent("spec", ohReport);
    core.subscribeEvent("suite", ohReport);
    core.subscribeEvent("task", ohReport);
    const configService = core.getDefaultService("config");
    if (abilityDelegatorArguments !== null) {
      let testParameters = configService.translateParams(
        abilityDelegatorArguments.parameters
      );
      console.info(`${TAG}parameters:${JSON.stringify(testParameters)}`);
      configService.setConfig(testParameters);
    }
    testsuite();
    core.execute(abilityDelegator);
  }
  static async hypiumInitWorkers(
    abilityDelegator,
    scriptURL,
    workerNum = 8,
    params
  ) {
    console.info(`${TAG}, hypiumInitWorkers call,${scriptURL}`);
    let workerPromiseArray = [];

    // 开始统计时间
    let startTime = await SysTestKit.getRealTime();
    for (let i = 0; i < workerNum; i++) {
      // 创建worker线程
      const workerPromise = Hypium.createWorkerPromise(scriptURL, i, params);
      workerPromiseArray.push(workerPromise);
    }
    const ret = {
      total: 0,
      failure: 0,
      error: 0,
      pass: 0,
      ignore: 0,
      duration: 0
    };
    Promise.all(workerPromiseArray)
      .then(async (items) => {
        console.info(
          `${TAG}, all result from workers, ${JSON.stringify(items)}`
        );
        let allItemList = new Array();
        // 统计执行结果
        Hypium.handleWorkerTestResult(ret, allItemList, items);
        console.info(`${TAG}, all it result, ${JSON.stringify(allItemList)}`);
        // 统计用例执行结果
        const retResult = {
          total: 0,
          failure: 0,
          error: 0,
          pass: 0,
          ignore: 0,
          duration: 0
        };
        // 标记用例执行结果
        Hypium.configWorkerItTestResult(retResult, allItemList);
        // 打印用例结果
        Hypium.printWorkerTestResult(abilityDelegator, allItemList);
        // 用例执行完成统计时间
        let endTime = await SysTestKit.getRealTime();
        const taskConsuming = endTime - startTime;
        const message =
          `\n${PrintTag.OHOS_REPORT_ALL_RESULT}: stream=Test run: runTimes: ${ret.total},total: ${retResult.total}, Failure: ${retResult.failure}, Error: ${retResult.error}, Pass: ${retResult.pass}, Ignore: ${retResult.ignore}` +
          `\n${PrintTag.OHOS_REPORT_ALL_CODE}: ${retResult.failure > 0 || retResult.error > 0 ? -1 : 0}` +
          `\n${PrintTag.OHOS_REPORT_ALL_STATUS}: taskconsuming=${taskConsuming > 0 ? taskConsuming : ret.duration}`;
        abilityDelegator.printSync(message);
        console.info(`${TAG}, [end] you worker test`);
        abilityDelegator.finishTest("you worker test finished!!!", 0, () => {});
      })
      .catch((e) => {
        console.info(
          `${TAG}, [end] error you worker test, ${JSON.stringify(e)}`
        );
        abilityDelegator.finishTest(
          "you worker test error finished!!!",
          0,
          () => {}
        );
      })
      .finally(() => {
        console.info(`${TAG}, all promise finally end`);
      });
  }
  // 创建worker线程
  static createWorkerPromise(scriptURL, i, params) {
    console.info(`${TAG}, createWorkerPromiser, ${scriptURL}, ${i}`);
    const workerPromise = new Promise((resolve, reject) => {
      const workerInstance = new worker.ThreadWorker(scriptURL, {
        name: `worker_${i}`
      });
      console.info(`${TAG}, send data to worker`);
      // 发送数据到worker线程中
      workerInstance.postMessage(params);
      workerInstance.onmessage = function (e) {
        let currentThreadName = e.data?.currentThreadName;
        console.info(
          `${TAG}, receview data from ${currentThreadName}, ${JSON.stringify(e.data)}`
        );
        //
        resolve(e.data?.summary);
        console.info(`${TAG}, ${currentThreadName} finish`);
        workerInstance.terminate();
      };
      workerInstance.onerror = function (e) {
        console.info(`${TAG}, worker error, ${JSON.stringify(e)}`);
        reject(e);
        workerInstance.terminate();
      };
      workerInstance.onmessageerror = function (e) {
        console.info(`${TAG}, worker message error, ${JSON.stringify(e)}`);
        reject(e);
        workerInstance.terminate();
      };
    });
    return workerPromise;
  }
  static handleWorkerTestResult(ret, allItemList, items) {
    console.info(`${TAG}, handleWorkerTestResult, ${JSON.stringify(items)}`);
    for (const {
      total,
      failure,
      error,
      pass,
      ignore,
      duration,
      itItemList
    } of items) {
      ret.total += total;
      ret.failure += failure;
      ret.error += error;
      ret.pass += pass;
      ret.ignore += ignore;
      ret.duration += duration;
      Hypium.handleItResult(allItemList, itItemList);
    }
  }
  static handleItResult(allItemList, itItemList) {
    // 遍历所有的用例结果统计最终结果
    for (const { currentThreadName, description, result } of itItemList) {
      let item = allItemList.find((it) => it.description === description);
      if (item) {
        let itResult = item.result;
        // 当在worker中出现一次failure就标记为failure, 出现一次error就标记为error, 所有线程都pass才标记为pass
        if (itResult === 0) {
          item.result = result;
          item.currentThreadName = currentThreadName;
        }
      } else {
        let it = {
          description: description,
          currentThreadName: currentThreadName,
          result: result
        };
        allItemList.push(it);
      }
    }
  }
  static configWorkerItTestResult(retResult, allItemList) {
    console.info(
      `${TAG}, configWorkerItTestResult, ${JSON.stringify(allItemList)}`
    );
    for (const { currentThreadName, description, result } of allItemList) {
      console.info(`${TAG}, description, ${description}, result,${result}`);
      retResult.total++;
      if (result === 0) {
        retResult.pass++;
      } else if (result === -1) {
        retResult.error++;
      } else if (result === -2) {
        retResult.failure++;
      } else {
        retResult.ignore++;
      }
    }
  }
  static printWorkerTestResult(abilityDelegator, allItemList) {
    console.info(
      `${TAG}, printWorkerTestResult, ${JSON.stringify(allItemList)}`
    );
    let index = 1;
    for (const { currentThreadName, description, result } of allItemList) {
      console.info(
        `${TAG}, description print, ${description}, result,${result}`
      );
      let itArray = description.split("#");
      let des;
      let itName;
      if (itArray.length > 1) {
        des = itArray[0];
        itName = itArray[1];
      } else if (itArray.length > 1) {
        des = itArray[0];
        itName = itArray[0];
      } else {
        des = "undefined";
        itName = "undefined";
      }

      let msg = `\n${PrintTag.OHOS_REPORT_WORKER_STATUS}: class=${des}`;
      msg += `\n${PrintTag.OHOS_REPORT_WORKER_STATUS}: test=${itName}`;
      msg += `\n${PrintTag.OHOS_REPORT_WORKER_STATUS}: current=${index}`;
      msg += `\n${PrintTag.OHOS_REPORT_WORKER_STATUS}: CODE=${result}`;
      abilityDelegator.printSync(msg);
      index++;
    }
  }
  static hypiumWorkerTest(
    abilityDelegator,
    abilityDelegatorArguments,
    testsuite,
    workerPort
  ) {
    console.info(`${TAG}, hypiumWorkerTest call`);
    SysTestKit.workerPort = workerPort;
    let currentWorkerName = workerPort.name;
    console.info(
      `${TAG}, hypiumWorkerTest_currentWorkerName: ${currentWorkerName}`
    );
    Hypium.hypiumTest(abilityDelegator, abilityDelegatorArguments, testsuite);
  }

  static registerAssert(customAssertion) {
    const core = Core.getInstance();
    const expectService = core.getDefaultService("expect");
    let matchers = {};
    matchers[customAssertion.name] = customAssertion;
    expectService.addMatchers(matchers);
    expectService.customMatchers.push(customAssertion.name);
    console.info(`${TAG}success to register the ${customAssertion.name}`);
  }

  static unregisterAssert(customAssertion) {
    const core = Core.getInstance();
    const expectService = core.getDefaultService("expect");
    let customAssertionName =
      typeof customAssertion === "function"
        ? customAssertion.name
        : customAssertion;
    expectService.removeMatchers(customAssertionName);
    console.info(`${TAG}success to unregister the ${customAssertionName}`);
  }
}

export {
  Hypium,
  Core,
  DEFAULT,
  TestType,
  Size,
  Level,
  DataDriver,
  ExpectExtend,
  OhReport,
  SysTestKit,
  describe,
  beforeAll,
  beforeEach,
  afterEach,
  afterAll,
  it,
  expect,
  beforeItSpecified,
  afterItSpecified,
  xdescribe,
  xit,
  MockKit,
  when,
  ArgumentMatchers
};
