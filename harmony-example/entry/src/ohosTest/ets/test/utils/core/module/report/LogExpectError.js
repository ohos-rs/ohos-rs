/*
 * Copyright (c) 2024 Huawei Device Co., Ltd.
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

class LogExpectError {
  static getErrorMsg(matcherName, actualValue, expect, originMsg) {
    if (matcherName === "assertNull") {
      return "expect not null, actualValue is " + actualValue;
    }
    if (matcherName === "assertTrue") {
      return "expect not true, actualValue is " + actualValue;
    }
    if (matcherName === "assertFalse") {
      return "expect not false, actualValue is " + actualValue;
    }
    if (matcherName === "assertEqual") {
      return (
        "expect not Equal, actualValue is " + actualValue + " equals " + expect
      );
    }
    if (matcherName === "assertContain") {
      return "expect not have, " + actualValue + " have " + expect;
    }
    if (matcherName === "assertInstanceOf") {
      return (
        "expect not InstanceOf, " +
        actualValue +
        " is " +
        Object.prototype.toString.call(actualValue) +
        expect
      );
    }
    if (matcherName === "assertLarger") {
      return "expect not Larger, " + actualValue + " is larger than " + expect;
    }
    if (matcherName === "assertLargerOrEqual") {
      return (
        "expect not LargerOrEqual, " + actualValue + " larger than " + expect
      );
    }
    if (matcherName === "assertLess") {
      return "expect not Less, " + actualValue + " less than " + expect;
    }
    if (matcherName === "assertLessOrEqual") {
      return (
        "expect not LessOrEqual, " + actualValue + " is less than " + expect
      );
    }
    if (matcherName === "assertNaN") {
      return "expect not NaN, actualValue is " + actualValue;
    }
    if (matcherName === "assertNegUnlimited") {
      return "expect not NegUnlimited, actualValue is " + actualValue;
    }
    if (matcherName === "assertPosUnlimited") {
      return "expect not PosUnlimited, actualValue is " + actualValue;
    }
    if (matcherName === "assertUndefined") {
      return "expect not Undefined, actualValue is " + actualValue;
    }
    return originMsg;
  }
}
export default LogExpectError;
