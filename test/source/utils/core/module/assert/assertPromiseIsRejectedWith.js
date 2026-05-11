/*
 * Copyright (c) 2022 Huawei Device Co., Ltd.
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

import isPromiseLike from "./isPromiseLike";

function assertPromiseIsRejectedWith(actualPromise, expectedValue) {
  if (!isPromiseLike(actualPromise)) {
    return Promise.reject().then(
      function () {},
      function () {
        return { pass: false, message: "Expected not be called on a promise." };
      },
    );
  }

  function tips(passed) {
    return (
      "Expected a promise " +
      (passed ? "not " : "") +
      "to be rejected with " +
      JSON.stringify(expectedValue[0])
    );
  }

  const expected = expectedValue?.[0];

  return actualPromise.then(
    function (got) {
      return {
        pass: false,
        message: tips(false) + " but actualValue is resolve",
      };
    },
    function (actualValue) {
      if (
        expected &&
        typeof expected === "object" &&
        (expected.message === undefined || actualValue?.message === expected.message) &&
        (expected.code === undefined || actualValue?.code === expected.code)
      ) {
        return {
          pass: true,
          message:
            "actualValue was rejected with " +
            JSON.stringify({ message: actualValue?.message, code: actualValue?.code }) +
            ".",
        };
      }

      if (typeof expected === "string" && actualValue?.message === expected) {
        return {
          pass: true,
          message: "actualValue was rejected with " + actualValue.message + ".",
        };
      }

      if (JSON.stringify(actualValue) == JSON.stringify(expected)) {
        return {
          pass: true,
          message: "actualValue was rejected with " + JSON.stringify(actualValue) + ".",
        };
      }

      return {
        pass: false,
        message:
          tips(false) +
          " but it was rejected with " +
          JSON.stringify({ message: actualValue?.message, code: actualValue?.code }) +
          ".",
      };
    },
  );
}

export default assertPromiseIsRejectedWith;
