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

import SysTestKit from "../kit/SysTestKit";
import { collectCoverageData } from "../coverage/coverageCollect";
import { TAG, PrintTag } from "../../Constant";

class OhReport {
  constructor(attr) {
    this.delegator = attr.delegator;
    this.abilityDelegatorArguments = attr.abilityDelegatorArguments;
    this.id = "report";
    this.index = 0;
    this.duration = 0;
    this.currentThreadName = "mainThread";
  }

  init(coreContext) {
    this.coreContext = coreContext;
    this.suiteService = this.coreContext.getDefaultService("suite");
    this.specService = this.coreContext.getDefaultService("spec");
    if (SysTestKit.workerPort !== null) {
      this.currentThreadName = SysTestKit.workerPort.name;
    }
  }

  taskStart() {}

  async taskDone() {
    let summary = this.suiteService.getSummary();
    if (this.abilityDelegatorArguments !== null) {
      this.taskDoneTime = new Date().getTime();
      const configService = this.coreContext.getDefaultService("config");
      const suiteService = this.coreContext.getDefaultService("suite");
      const specService = this.coreContext.getDefaultService("spec");
      if (configService["coverage"] === "true") {
        await collectCoverageData();
      }
      let message =
        "\n" +
        `${PrintTag.OHOS_REPORT_RESULT}: stream=Tests run: ` +
        summary.total +
        ", Failure: " +
        summary.failure;
      message += ", Error: " + summary.error;
      message += ", Pass: " + summary.pass;
      message += ", Ignore: " + summary.ignore;
      if (specService.skipSpecNum > 0) {
        message += ", SkipSpec: " + specService.skipSpecNum;
      }
      message +=
        "\n" +
        `${PrintTag.OHOS_REPORT_CODE}: ` +
        (summary.failure > 0 ? -1 : 0) +
        "\n";
      let isHasError = summary.failure > 0 || summary.error > 0;
      let config = this.coreContext.getDefaultService("config");
      if (config.isBreakOnError() && isHasError) {
        // 未执行全部说明
        message +=
          "\n" +
          `${PrintTag.OHOS_REPORT_RESULT}: breakOnError model, Stopping whole test suite if one specific test case failed or error` +
          "\n";
      }
      message +=
        `${PrintTag.OHOS_REPORT_STATUS}: taskconsuming=` +
        summary.duration +
        "\n";
      console.info(`${message}`);
      await SysTestKit.print(message);
    }
    if (SysTestKit.workerPort === null || SysTestKit.workerPort === undefined) {
      // 主线程执行完成 结束任务。
      console.info(`${TAG}report print success`);
      this.delegator.finishTest("your test finished!!!", 0, () => {});
    } else {
      // worker线程执行完成将数据发送到主线程中。
      let sendData = {
        currentThreadName: this.currentThreadName,
        summary: summary
      };
      console.info(
        `${TAG}, send data to mainThread, ${this.currentThreadName}, ${JSON.stringify(sendData)}`
      );
      SysTestKit.workerPort.postMessage(sendData);
    }
  }

  incorrectFormat() {
    if (this.coreContext.getDefaultService("config").filterValid.length !== 0) {
      var value = this.coreContext.getDefaultService("config").filterValid;
      var message = "this param " + value.join(",") + " is invalid" + "\n";
      this.delegator.finishTest(message, 0, () => {});
    }
  }

  incorrectTestSuiteFormat() {
    if (
      this.coreContext.getDefaultService("config").filterXdescribe.length !== 0
    ) {
      let value = this.coreContext.getDefaultService("config").filterXdescribe;
      let message =
        "xdescribe " + value.join(",") + " should not contain it" + "\n";
      this.delegator.finishTest(message, 0, () => {});
    }
  }
  async suiteStart() {
    if (this.abilityDelegatorArguments !== null) {
      let specArr = [];
      this.suiteService.getAllChildSuiteNum(
        this.suiteService.getCurrentRunningSuite(),
        specArr
      );
      let message = "\n" + `${PrintTag.OHOS_REPORT_SUM}: ` + specArr.length;
      this.suiteService.setCurrentRunningSuiteDesc(
        this.suiteService.getRootSuite(),
        this.suiteService.getCurrentRunningSuite(),
        ""
      );
      message +=
        "\n" +
        `${PrintTag.OHOS_REPORT_STATUS}: class=` +
        this.suiteService.getCurrentRunningSuiteDesc() +
        "\n";
      if (this.suiteService.currentRunningSuite.isSkip) {
        message +=
          `${PrintTag.OHOS_REPORT_STATUS}: skipReason=` +
          this.suiteService.currentRunningSuite.skipReason +
          "\n";
      }
      if (SysTestKit.workerPort !== null) {
        message +=
          `${PrintTag.OHOS_REPORT_STATUS}: currentWorkerName=` +
          this.currentThreadName;
      }
      console.info(`${message}`);
      await SysTestKit.print(message);
      console.info(
        `${TAG}${this.suiteService.getCurrentRunningSuite().description} suiteStart print success`
      );
    }
  }

  async suiteDone() {
    if (this.abilityDelegatorArguments !== null) {
      const currentRunningSuite = this.suiteService.getCurrentRunningSuite();
      this.suiteService.setCurrentRunningSuiteDesc(
        this.suiteService.getRootSuite(),
        this.suiteService.getCurrentRunningSuite(),
        ""
      );
      let message =
        "\n" +
        `${PrintTag.OHOS_REPORT_STATUS}: class=` +
        this.suiteService.getCurrentRunningSuiteDesc();
      if (
        this.suiteService.currentRunningSuite.isSkip &&
        this.suiteService.currentRunningSuite.skipReason !== ""
      ) {
        message +=
          "\n" +
          `${PrintTag.OHOS_REPORT_STATUS}: skipReason=` +
          this.suiteService.currentRunningSuite.skipReason;
      }
      message +=
        "\n" +
        `${PrintTag.OHOS_REPORT_STATUS}: suiteconsuming=` +
        this.suiteService.getCurrentRunningSuite().duration;
      if (currentRunningSuite.hookError) {
        message +=
          "\n" +
          `${PrintTag.OHOS_REPORT_STATUS}: ${currentRunningSuite.hookError.message}`;
      }
      message += "\n";
      if (SysTestKit.workerPort !== null) {
        message +=
          `${PrintTag.OHOS_REPORT_STATUS}: currentWorkerName=` +
          this.currentThreadName;
      }
      console.info(`${message}`);
      await SysTestKit.print(message);
      console.info(
        `${TAG}${this.suiteService.getCurrentRunningSuite().description} suiteDone print success`
      );
    }
  }

  async specStart() {
    if (this.abilityDelegatorArguments !== null) {
      let message =
        "\n" +
        `${PrintTag.OHOS_REPORT_STATUS}: class=` +
        this.suiteService.getCurrentRunningSuiteDesc();
      message +=
        "\n" + `${PrintTag.OHOS_REPORT_STATUS}: current=` + ++this.index;
      message += "\n" + `${PrintTag.OHOS_REPORT_STATUS}: id=JS`;
      message +=
        "\n" +
        `${PrintTag.OHOS_REPORT_STATUS}: numtests=` +
        this.specService.getTestTotal();
      message += "\n" + `${PrintTag.OHOS_REPORT_STATUS}: stream=`;
      message +=
        "\n" +
        `${PrintTag.OHOS_REPORT_STATUS}: test=` +
        this.specService.currentRunningSpec.description;
      message += "\n" + `${PrintTag.OHOS_REPORT_STATUS_CODE}: 1` + "\n";
      if (this.specService.currentRunningSpec.isSkip) {
        message +=
          `${PrintTag.OHOS_REPORT_STATUS}: skipReason=` +
          this.specService.currentRunningSpec.skipReason +
          "\n";
      }
      if (SysTestKit.workerPort !== null) {
        message +=
          `${PrintTag.OHOS_REPORT_STATUS}: currentWorkerName=` +
          this.currentThreadName;
      }
      console.info(`${message}`);
      await SysTestKit.print(message);
      console.info(
        `${TAG}${this.specService.currentRunningSpec.description} specStart start print success`
      );
    }
  }

  async specDone() {
    if (this.abilityDelegatorArguments !== null) {
      let message =
        "\n" +
        `${PrintTag.OHOS_REPORT_STATUS}: class=` +
        this.suiteService.getCurrentRunningSuiteDesc();
      message += "\n" + `${PrintTag.OHOS_REPORT_STATUS}: current=` + this.index;
      message += "\n" + `${PrintTag.OHOS_REPORT_STATUS}: id=JS`;
      message +=
        "\n" +
        `${PrintTag.OHOS_REPORT_STATUS}: numtests=` +
        this.specService.getTestTotal();
      let messageStack = "";
      let messageCode = "";
      if (this.specService.currentRunningSpec.error) {
        messageStack =
          `${PrintTag.OHOS_REPORT_STATUS}: stack=` +
          this.specService.currentRunningSpec.error?.stack?.slice(0, -1);
        messageCode += `${PrintTag.OHOS_REPORT_STATUS}: stream=`;
        messageCode +=
          this.specService.currentRunningSpec.expectMsg !== ""
            ? `message: ${this.specService.currentRunningSpec.expectMsg}, Error in ${this.specService.currentRunningSpec.description}, ${this.specService.currentRunningSpec.error?.message}`
            : `Error in ${this.specService.currentRunningSpec.description}, ${this.specService.currentRunningSpec.error?.message}`;
        messageCode +=
          "\n" +
          `${PrintTag.OHOS_REPORT_STATUS}: test=` +
          this.specService.currentRunningSpec.description;
        messageCode += "\n" + `${PrintTag.OHOS_REPORT_STATUS_CODE}: -1` + "\n";
      } else if (this.specService.currentRunningSpec) {
        if (this.specService.currentRunningSpec.fail) {
          messageStack +=
            `${PrintTag.OHOS_REPORT_STATUS}: stack=` +
            this.specService.currentRunningSpec.fail?.stack?.slice(0, -1);
          messageCode += `${PrintTag.OHOS_REPORT_STATUS}: stream=`;
          messageCode +=
            this.specService.currentRunningSpec.expectMsg !== ""
              ? `message: ${this.specService.currentRunningSpec.expectMsg}, Error in ${this.specService.currentRunningSpec.description}, ${this.specService.currentRunningSpec.fail?.message}`
              : `Error in ${this.specService.currentRunningSpec.description}, ${this.specService.currentRunningSpec.fail?.message}`;
          messageCode +=
            "\n" +
            `${PrintTag.OHOS_REPORT_STATUS}: test=` +
            this.specService.currentRunningSpec.description;
          messageCode +=
            "\n" + `${PrintTag.OHOS_REPORT_STATUS_CODE}: -2` + "\n";
        } else {
          messageStack += `${PrintTag.OHOS_REPORT_STATUS}: stream=`;
          messageCode +=
            `${PrintTag.OHOS_REPORT_STATUS}: test=` +
            this.specService.currentRunningSpec.description;
          messageCode += "\n" + `${PrintTag.OHOS_REPORT_STATUS_CODE}: 0` + "\n";
          messageCode += this.specService.currentRunningSpec.isSkip
            ? `${PrintTag.OHOS_REPORT_STATUS}: skipReason=` +
              this.specService.currentRunningSpec.skipReason +
              "\n"
            : "";
        }
      } else {
        messageCode += "\n";
      }
      messageCode +=
        `${PrintTag.OHOS_REPORT_STATUS}: consuming=` +
        this.specService.currentRunningSpec.duration +
        "\n";
      if (SysTestKit.workerPort !== null) {
        messageCode +=
          `${PrintTag.OHOS_REPORT_STATUS}: currentWorkerName=` +
          this.currentThreadName;
      }
      console.info(`${message}`);
      console.info(`\n${messageStack}`);
      console.info(`\n${messageCode}`);
      await SysTestKit.print(message);
      await SysTestKit.print(messageStack);
      await SysTestKit.print(messageCode);
      console.info(
        `${TAG}${this.specService.currentRunningSpec.description} specDone end print success`
      );
    }
  }
}

export default OhReport;
