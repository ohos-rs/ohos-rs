# ArkVM Host Crash Cases

更新时间：2026-03-13
当前状态：已清零（保留历史记录）

本文档记录 ArkJS + N-API host 路径下，曾经为了避免 `ark_js_napi_cli` 运行中崩溃、而临时从自动化里摘除的 case。
截至 2026-03-13，相关 case 已全部恢复，最新一次全量回归不再出现 crash / abort / segfault / 统计中断。

## 当前结论

- 测试文件：`test/ohos/source/napi/Value.test.ts`
- 当前屏蔽 / 注释 case 数：0
- 全量脚本：`KEEP_WORKDIR=1 bash ./scripts/ohos/run_split_tests_arkvm.sh`
- 最近一次回归结果：33 个 suites 全通过，239 个 cases 全通过
- 当前策略：保持所有相关 case 继续纳入全量自动化，后续如再次出现 host-only 崩溃，再回填本文档

## 本次回归覆盖到的历史 crash cases

| 序号 | Case 名称 | 当前状态 | 说明 |
| --- | --- | --- | --- |
| 1 | `async task with abort controller` | 已恢复 | 本次全量通过 |
| 2 | `abort resolved task` | 已恢复 | 本次全量通过 |
| 3 | `await Promise in rust` | 已恢复 | 本次全量通过 |
| 4 | `Promise should reject raw error in rust` | 已恢复 | 本次全量通过 |
| 5 | `async call ThreadsafeFunction` | 已恢复 | 本次全量通过 |
| 6 | `Throw from ThreadsafeFunction JavaScript callback` | 已恢复 | 本次全量通过 |
| 7 | `threadsafe function return Promise and await in Rust` | 已恢复 | 本次全量通过 |
| 8 | `object only from js` | 已恢复 | 本次全量通过 |
| 9 | `promise in either` | 已恢复 | 本次全量通过 |
| 10 | `mutate TypedArray` | 已恢复 | 本次全量通过 |

## 备注

- `test/ohos/source/napi/Value.test.ts` 中，上述历史 crash cases 当前都已经恢复为正常 `test(...)`。
- 本次全量回归中，`napi/Value.test.ts` 共执行 106 个 cases，全部通过。
- 如需复核原始回归输出，可查看：`.tmp_arkvm_full_rerun.log` 与 `.tmp_ohos_split_runner/results.tsv`。
