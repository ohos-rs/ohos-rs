/*
 * Copyright (c) 2023 Huawei Device Co., Ltd.
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

import SysTestKit from '../kit/SysTestKit';
import fs from '@ohos.file.fs';
import {TAG} from '../../Constant';

const jsCoverageFileName = 'js_coverage.json';

export async function collectCoverageData() {
    if (globalThis.__coverage__ === undefined) {
        console.info(`${TAG} globalThis not have coverage`);
        return;
    }
    const strJson = JSON.stringify(globalThis.__coverage__);
    let testMode = globalThis.__testMode__;
    console.info(`${TAG} coverage data testMode: ${testMode}`);
    let savePath = globalThis.__savePath__;
    console.info(`${TAG} write coverage data to: ${savePath}`);
    let readPath = globalThis.__readPath__;
    console.info(`${TAG} read coverage data in: ${readPath}`);

    // run callback mode if local test or (save path and read path ) is not defined
    if (!testMode || !isCoveragePathValid(savePath)) {
        console.info(`${TAG} run coverage data in call back mode`);
        const strLen = strJson.length;
        const maxLen = 500;
        const maxCount = Math.floor(strLen / maxLen);
        const OHOS_REPORT_COVERAGE_DATA = 'OHOS_REPORT_COVERAGE_DATA:';
        for (let count = 0; count <= maxCount; count++) {
            console.info(`${OHOS_REPORT_COVERAGE_DATA} ${strJson.substring(count * maxLen, (count + 1) * maxLen)}`);
            await SysTestKit.print(`${OHOS_REPORT_COVERAGE_DATA} ${strJson.substring(count * maxLen, (count + 1) * maxLen)}`);
        }
        return;
    }
    console.info(`${TAG} run coverage data in save file mode`);
    if (fs.accessSync(savePath)) {
        fs.unlinkSync(savePath);
    }
    
    let inputPathDir = savePath.substring(0, savePath.length - jsCoverageFileName.length);
    if (!fs.accessSync(inputPathDir)) {
        console.info(`${TAG} coverage data create dir: ${inputPathDir}`);
        fs.mkdirSync(inputPathDir);
    }

    let file = fs.openSync(savePath, fs.OpenMode.READ_WRITE | fs.OpenMode.CREATE);
    let writeLen = fs.writeSync(file.fd, strJson, {encoding:'utf-8'});
    console.info(`${TAG} write coverage data success: ${writeLen}`);
    fs.closeSync(file);
    const OHOS_REPORT_COVERAGE_PATH = 'OHOS_REPORT_COVERAGE_PATH:';
    await SysTestKit.print(`${OHOS_REPORT_COVERAGE_PATH} ${readPath}`);
    console.info(`${OHOS_REPORT_COVERAGE_PATH} ${readPath}`);
}

function isCoveragePathValid(inputPath) {
    if (!inputPath) {
        return false;
    }
    if (inputPath.indexOf(jsCoverageFileName) === -1) {
        return false;
    }
    return true;
}