# ArkTS + N-API 测试入口

当前仓库只保留一套 ArkTS + N-API 自动化测试链路，入口如下：

- 核心执行脚本：`scripts/ohos/run_split_tests_arkvm.sh`
- 测试拆分脚本：`scripts/ohos/split_ohos_tests.sh`
- CI 包装脚本：`scripts/ci/run_ets_napi_host_tests.sh`

## 用法

本地直跑：

```bash
export ARK_HOST_TOOLS_DIR=/Users/ranger/Desktop/x64_linux_static
just unit-test-docker
```

CI/脚本模式：

```bash
bash ./scripts/ci/run_ets_napi_host_tests.sh
```

## 约定

- `ARK_HOST_TOOLS_DIR` 指向 Linux x64 的 ArkJS N-API host bundle。
- 动态库加载基于 `ark_js_napi_cli` + `requireNapiPreview(...)`。
- `example` / `compact` 会以 `arkvm-test` feature 构建，用于把 `target_env = "ohos"` 相关代码编进宿主测试产物。
- 实际执行依赖 `libace_napi.so`，由 host bundle 提供。

## 保留这些脚本的原因

- `run_split_tests_arkvm.sh`：唯一真实执行器，负责构建、拆分、编译 `.abc`、加载 `.so`、汇总结果。
- `split_ohos_tests.sh`：把 Harmony 示例中的 Hypium 风格用例拆成可独立执行的 ArkTS suite。
- `run_ets_napi_host_tests.sh`：给 CI 提供一个可跳过的稳定入口，未配置 host bundle 时直接退出，不报假失败。
