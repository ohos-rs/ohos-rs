# ArkVM Test 失败 Suite 分组

更新时间：2026-03-08
运行命令：`export ARK_HOST_TOOLS_DIR=/Users/ranger/Desktop/x64_linux_static; export KEEP_WORKDIR=1; just unit-test-docker`
结果文件：`.tmp_ohos_split_runner/results.tsv`

## 当前结论

- suite 总数：32
- suite 通过：15
- suite 失败：17
- 已统计 case 总数：130
- case 通过：96
- case 失败：18
- case error：16
- case 通过率：约 73.8%
- 说明：`napi/Value.test.ts` 在中途 abort，文件内大量 case 未进入最终统计，因此当前通过率偏乐观，不能视为稳定达到 80%~90%。

## 分类说明

这里按“主因”归类，而不是按“唯一原因”归类。

- `环境不匹配`：测试依赖的 cwd、环境变量、文件路径、事件循环/宿主行为，与当前 Docker + ark_js_napi_cli 环境不一致。
- `N-API 实现缺口`：更像是当前 `arkvm-test` 路径下的 N-API/绑定能力、导出结果、错误映射、对象行为与预期不一致。
- `测试假设问题`：更像是断言方式、对象比较方式、深比较规则与当前 runner/运行时对象表示不一致；不一定是底层能力真的错。

## 一、环境不匹配

### 1. `compat-mode/Env.test.ts`
- 主判断：环境不匹配
- 现象：访问环境变量时报错 `NotPresent`
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_Env_test.log`
- 关键日志：`called Result::unwrap() on an Err value: NotPresent`
- 判断依据：当前运行容器/CLI 环境没有提供测试预期中的环境变量，属于宿主环境差异。

### 2. `compat-mode/Global.test.ts`
- 主判断：环境不匹配
- 现象：`should setTimeout` 断言 `expect 0 equals 1`
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_Global_test.log`
- 判断依据：更像是当前 ArkJS CLI 下定时器调度/事件循环推进时机与原测试假设不一致，而不是某个导出的 N-API 函数返回错值。

### 3. `compat-mode/napi4/TokioRT.test.ts`
- 主判断：环境不匹配
- 现象：多个 case 报 `No such file or directory (os error 2)`
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_napi4_TokioRT_test.log`
- 关键日志：`failed to read file, No such file or directory (os error 2)`
- 判断依据：测试依赖的输入文件在当前 split runner 工作目录下不存在，优先看作测试资源路径/工作目录不匹配。

### 4. `compat-mode/napi4/TokioReadFile.test.ts`
- 主判断：环境不匹配
- 现象：原本期望 `null` 错误，实际收到 `No such file or directory`
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_napi4_TokioReadFile_test.log`
- 判断依据：与上面相同，优先是文件路径和工作目录布局问题。

### 5. `napi/Value.test.ts`
- 主判断：环境不匹配
- 现象：`callback` case 中预期 cwd 为 `/`，实际是 `/work/.tmp_ohos_split_runner/workspace`，随后 native `unwrap()` 导致整套 abort
- 证据：`.tmp_ohos_split_runner/logs/napi_Value_test.log`
- 关键日志：`AssertException: expect /work/.tmp_ohos_split_runner/workspace equals /`
- 次级问题：`examples/napi/src/callback.rs` 使用了 `callback(...).unwrap()`，把 JS 断言失败放大成进程崩溃
- 判断依据：根因先是测试假设 cwd 为 `/`，与当前 Docker 工作目录不一致；panic 是放大器，不是最初触发点。

## 二、N-API 实现缺口

### 6. `compat-mode/Class.test.ts`
- 主判断：N-API 实现缺口
- 现象：
  - `should be able to re-create wrapped native value` 报 wrapped type 不匹配
  - `should be able to new class instance in native side` 报 `undefined is not callable`
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_Class_test.log`
- 判断依据：更像是类实例重绑、native 构造导出、wrap/unwrap 类型关系没有对齐预期。

### 7. `compat-mode/CleanEnv.test.ts`
- 主判断：N-API 实现缺口
- 现象：`add/remove cleanup hook` 都失败
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_CleanEnv_test.log`
- 判断依据：cleanup hook 属于 Node-API 生命周期能力，失败点集中且稳定，优先认为当前能力缺口。

### 8. `compat-mode/CreateExternal.test.ts`
- 主判断：N-API 实现缺口
- 现象：`add/remove cleanup hook` 都失败
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_CreateExternal_test.log`
- 判断依据：与 cleanup hook / external value 生命周期能力直接相关。

### 9. `compat-mode/JsValue.test.ts`
- 主判断：N-API 实现缺口
- 现象：
  - `instanceof` 直接报错
  - `cast_unknown` 结果不符合预期
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_JsValue_test.log`
- 判断依据：更像是值包装、类型判断、对象转换行为没有对齐。

### 10. `compat-mode/Object.test.ts`
- 主判断：N-API 实现缺口
- 现象：
  - `testHasOwnProperty`
  - `testHasOwnPropertyJs`
  - `testGetPrototype`
 失败
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_Object_test.log`
- 判断依据：集中在对象属性归属、原型链行为，属于典型对象语义差异。

### 11. `compat-mode/Spawn.test.ts`
- 主判断：N-API 实现缺口
- 现象：两个核心 case 都报 `undefined is not callable`
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_Spawn_test.log`
- 补充：`results.tsv` 中该 suite 还有 `no_napi_calls` 标记
- 判断依据：更像是相关导出没有生成或未成功注册，而不是环境目录问题。

### 12. `compat-mode/napi4/Deferred.test.ts`
- 主判断：N-API 实现缺口
- 现象：拒绝 promise 时，预期 `{"message":"Fail"}`，实际只得到 `{"code":"GenericFailure"}`
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_napi4_Deferred_test.log`
- 判断依据：错误对象映射、reject 路径上的信息保真度不足。

### 13. `napi/Genertor.test.ts`
- 主判断：N-API 实现缺口
- 现象：6 个 case 全 `ERROR`
- 证据：`.tmp_ohos_split_runner/logs/napi_Genertor_test.log`
- 关键日志：
  - `Cannot read property next of undefined`
  - `JSProxy::GetProperty: TypeError of trapResult`
- 判断依据：generator / iterator / proxy 相关返回对象行为明显不符合预期，优先看作实现缺口。

### 14. `napi/Stricts.test.ts`
- 主判断：N-API 实现缺口
- 现象：
  - `should validate External` 报 `ProxyCreate: target is not Object`
  - `should validate promise` 错误对象字段不完整
- 证据：`.tmp_ohos_split_runner/logs/napi_Stricts_test.log`
- 判断依据：一部分是 external 包装行为问题，一部分是错误对象结构问题，整体仍偏实现缺口。

## 三、测试假设问题

### 15. `compat-mode/Buffer.test.ts`
- 主判断：测试假设问题
- 现象：多个失败日志呈现出明显的“值看起来相同但 deepEqual 失败”模式
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_Buffer_test.log`
- 关键日志：
  - `10 is not deep equal 10`
  - `3 is not deep equal 3`
- 判断依据：这更像是 runner 当前 `deepEqual` 对 `Buffer` / typed array / borrowed buffer 的比较逻辑不完整。
- 备注：其中 `should create empty buffer` 的 `Failed to create buffer slice from data` 也提示底层实现可能仍有问题，所以这是“以测试假设为主的混合问题”。

### 16. `compat-mode/napi5/Date.test.ts`
- 主判断：测试假设问题
- 现象：日志中左右两侧日期字符串完全相同，但 `deepEqual` 仍失败
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_napi5_Date_test.log`
- 关键日志：`Sun Mar 08 2026 03:52:56 GMT+0000 is not deep equal Sun Mar 08 2026 03:52:56 GMT+0000`
- 判断依据：高度像 `Date` 对象的比较规则与 runner 的 `deepEqual` 不匹配，而不是日期值真的错了。

### 17. `compat-mode/serde/Ser.test.ts`
- 主判断：测试假设问题
- 现象：`[object Object] is not deep equal [object Object]`
- 证据：`.tmp_ohos_split_runner/logs/compat_mode_serde_Ser_test.log`
- 判断依据：优先怀疑对象深比较/序列化后对象表示与 runner 断言能力不一致，需要先确认字段级差异再决定是否真是底层序列化 bug。

## 建议的排查顺序

### P0：先修复会影响统计真实性的问题
- `napi/Value.test.ts`
- 目标：先去掉 native `unwrap()` 导致的进程 abort，让 `Value.test.ts` 的 100+ case 能完整跑完并出真实统计。

### P1：先修环境不匹配，通常回报最高
- `compat-mode/Env.test.ts`
- `compat-mode/napi4/TokioRT.test.ts`
- `compat-mode/napi4/TokioReadFile.test.ts`
- `napi/Value.test.ts` 中 cwd 相关假设
- `compat-mode/Global.test.ts`

### P2：集中修对象/类/外部值能力
- `compat-mode/Class.test.ts`
- `compat-mode/JsValue.test.ts`
- `compat-mode/Object.test.ts`
- `napi/Stricts.test.ts`
- `compat-mode/CreateExternal.test.ts`
- `compat-mode/CleanEnv.test.ts`

### P3：最后再看断言框架兼容性
- `compat-mode/Buffer.test.ts`
- `compat-mode/napi5/Date.test.ts`
- `compat-mode/serde/Ser.test.ts`

## 备注

本文档基于当前日志做“主因归类”。
其中以下 suite 带有明显混合属性，不建议一次性下最终结论：

- `compat-mode/Buffer.test.ts`
- `compat-mode/JsValue.test.ts`
- `compat-mode/Object.test.ts`
- `napi/Stricts.test.ts`
- `napi/Value.test.ts`

这些 suite 最好在修完 P0/P1 后重新跑一轮，再决定是否调整归类。
