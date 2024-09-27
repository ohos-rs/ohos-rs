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

const SUITES_KEY = "suites";
const SPECS_KEY = "items";
const DESCRIBE_KEY = "describe";
const IT_KEY = "it";
const PARAMS_KEY = "params";
const STRESS_KEY = "stress";

class ObjectUtils {
  static get(object, name, defaultValue) {
    let result = defaultValue;
    for (const key in object) {
      if (key === name) {
        return object[key];
      }
    }
    return result;
  }

  static has(object, key) {
    return Object.prototype.hasOwnProperty.call(object, key);
  }
}

class DataDriver {
  constructor(attr) {
    this.id = "dataDriver";
    this.data = attr.data || {};
  }

  init(coreContext) {
    this.coreContext = coreContext;
    this.suiteService = this.coreContext.getDefaultService("suite");
    this.specService = this.coreContext.getDefaultService("spec");
  }

  getSpecParamsValue(specs) {
    let specParams = [];
    let specDesc = this.specService.getCurrentRunningSpec().description;
    if (specs === null || specs === undefined) {
      return specParams;
    }
    for (const specItem of specs) {
      if (
        ObjectUtils.has(specItem, IT_KEY) &&
        ObjectUtils.get(specItem, IT_KEY) === specDesc
      ) {
        return ObjectUtils.get(specItem, PARAMS_KEY, specParams);
      }
    }
    return specParams;
  }

  getSpecParams() {
    let specParams = [];
    let suiteDesc = this.suiteService.getCurrentRunningSuite().description;
    let specDesc = this.specService.getCurrentRunningSpec().description;
    let suites = ObjectUtils.get(this.data, SUITES_KEY, []);
    for (const suiteItem of suites) {
      let describeValue = ObjectUtils.get(suiteItem, DESCRIBE_KEY, "");
      if (
        ObjectUtils.has(suiteItem, DESCRIBE_KEY) &&
        typeof describeValue === "object" &&
        describeValue.constructor === Array &&
        describeValue.includes(suiteDesc)
      ) {
        let specs = ObjectUtils.get(suiteItem, SPECS_KEY, []);
        return this.getSpecParamsValue(specs);
      }
    }
    return specParams;
  }

  getSuiteParams() {
    let suiteParams = {};
    let suiteDesc = this.suiteService.getCurrentRunningSuite().description;
    let suites = ObjectUtils.get(this.data, SUITES_KEY, []);
    for (const suiteItem of suites) {
      let describeValue = ObjectUtils.get(suiteItem, DESCRIBE_KEY, []);
      if (
        ObjectUtils.has(suiteItem, DESCRIBE_KEY) &&
        typeof describeValue === "object" &&
        describeValue.constructor === Array &&
        describeValue.includes(suiteDesc)
      ) {
        suiteParams = Object.assign(
          {},
          suiteParams,
          ObjectUtils.get(suiteItem, PARAMS_KEY, suiteParams)
        );
      }
    }
    return suiteParams;
  }

  getStressNum(specs, specDesc) {
    let stress = 1;
    if (specs === null || specs === undefined) {
      return stress;
    }
    for (const specItem of specs) {
      if (
        ObjectUtils.has(specItem, IT_KEY) &&
        ObjectUtils.get(specItem, IT_KEY) === specDesc
      ) {
        let tempStress = ObjectUtils.get(specItem, STRESS_KEY, stress);
        return Number.isInteger(tempStress) && tempStress >= 1
          ? tempStress
          : stress;
      }
    }
    return stress;
  }

  getSpecStress(specDesc) {
    let stress = 1;
    let suiteDesc = this.suiteService.getCurrentRunningSuite().description;
    let suites = ObjectUtils.get(this.data, SUITES_KEY, []);
    for (const suiteItem of suites) {
      let describeValue = ObjectUtils.get(suiteItem, DESCRIBE_KEY, "");
      if (
        ObjectUtils.has(suiteItem, DESCRIBE_KEY) &&
        typeof describeValue === "object" &&
        describeValue.constructor === Array &&
        describeValue.includes(suiteDesc)
      ) {
        let specs = ObjectUtils.get(suiteItem, SPECS_KEY, []);
        return this.getStressNum(specs, specDesc);
      }
    }
    return stress;
  }

  getSuiteStress(suiteDesc) {
    let stress = 1;
    let suites = ObjectUtils.get(this.data, SUITES_KEY, []);
    for (const suiteItem of suites) {
      let describeValue = ObjectUtils.get(suiteItem, DESCRIBE_KEY, []);
      if (
        ObjectUtils.has(suiteItem, DESCRIBE_KEY) &&
        typeof describeValue === "object" &&
        describeValue.constructor === Array &&
        describeValue.includes(suiteDesc)
      ) {
        let tempStress = ObjectUtils.get(suiteItem, STRESS_KEY, stress);
        return Number.isInteger(tempStress) && tempStress >= 1
          ? tempStress
          : stress;
      }
    }
    return stress;
  }
}

export default DataDriver;
