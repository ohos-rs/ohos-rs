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

import { LEVEL, SIZE, TESTTYPE } from "../../Constant";

class ClassFilter {
  constructor(suiteName, itName, params) {
    this.suiteName = suiteName;
    this.itName = itName;
    this.params = params;
  }

  filterSuite() {
    return !this.params
      .split(",")
      .map((item) => item.split("#")[0])
      .map((item) => item == this.suiteName)
      .reduce((pre, cur) => pre || cur, false);
  }

  filterIt() {
    let classArray = this.params.split(",") || [];
    let suiteFilterResult = classArray
      .filter((item) => !item.includes("#"))
      .map((item) => item == this.suiteName)
      .reduce((pre, cur) => pre || cur, false);
    let itFilterResult = classArray
      .filter((item) => item.includes("#"))
      .map((item) => item == this.suiteName + "#" + this.itName)
      .reduce((pre, cur) => pre || cur, false);
    return !(suiteFilterResult || itFilterResult);
  }
}

class NotClassFilter {
  constructor(suiteName, itName, params) {
    this.suiteName = suiteName;
    this.itName = itName;
    this.params = params;
  }

  filterSuite() {
    return this.params
      .split(",")
      .map((item) => item == this.suiteName)
      .reduce((pre, cur) => pre || cur, false);
  }

  filterIt() {
    return this.params
      .split(",")
      .some((item) => item == this.suiteName + "#" + this.itName);
  }
}

class SuiteAndItNameFilter {
  constructor(suiteName, itName, params) {
    this.suiteName = suiteName;
    this.itName = itName;
    this.params = params;
  }

  filterSuite() {
    return !this.params
      .split(",")
      .map((item) => item == this.suiteName)
      .reduce((pre, cur) => pre || cur, false);
  }

  filterIt() {
    return !this.params
      .split(",")
      .map((item) => item == this.itName)
      .reduce((pre, cur) => pre || cur, false);
  }
}

class TestTypesFilter {
  constructor(suiteName, itName, fi, params) {
    this.suiteName = suiteName;
    this.itName = itName;
    this.params = params;
    this.fi = fi;
  }

  filterIt() {
    return !(this.params === (this.fi & this.params) || this.fi === 0);
  }
}

class NestFilter {
  filterNestName(targetSuiteArray, targetSpecArray, suiteStack, desc) {
    let targetSuiteName = "";
    for (let key in suiteStack) {
      targetSuiteName = targetSuiteName + "." + suiteStack[key].description;
    }
    targetSuiteName = targetSuiteName.substring(2);
    const targetSpecName = targetSuiteName + "#" + desc;
    let isFilter = true;
    if (targetSpecArray.includes(targetSpecName)) {
      return false;
    }
    for (let index in targetSuiteArray) {
      if (targetSuiteName.startsWith(targetSuiteArray[index])) {
        return false;
      }
    }
    return isFilter;
  }

  filterNotClass(notClass, suiteStack, desc) {
    let filterNotClass = false;
    if (notClass != null) {
      let notClassArray = notClass.split(",");
      let targetSuiteName = "";
      for (let key in suiteStack) {
        targetSuiteName = targetSuiteName + "." + suiteStack[key].description;
      }
      targetSuiteName = targetSuiteName.substring(2);
      const targetSpecName = targetSuiteName + "#" + desc;
      if (
        notClassArray.includes(targetSpecName) ||
        notClassArray.some((key) => targetSpecName.startsWith(key))
      ) {
        filterNotClass = true;
      }
    }
    return filterNotClass;
  }

  filterLevelOrSizeOrTestType(level, size, testType, filter) {
    let result = false;
    if (filter === 0 || filter === "0") {
      return result;
    }
    if (level == null && size == null && testType == null) {
      return result;
    }
    if (level != null) {
      let levelFilter = LEVEL[`${level}`];
      result = result || filter === levelFilter;
    }
    if (size != null) {
      let sizeFilter = SIZE[`${size}`];
      result = result || filter === sizeFilter;
    }
    if (testType != null) {
      let testTypeFilter = TESTTYPE[`${testType}`];
      result = result || filter === testTypeFilter;
    }
    return !result;
  }
}
export {
  ClassFilter,
  NotClassFilter,
  SuiteAndItNameFilter,
  TestTypesFilter,
  NestFilter
};
