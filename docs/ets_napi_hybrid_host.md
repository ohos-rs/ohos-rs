# ArkTS + N-API 测试入口

当前仓库只保留一套 ArkTS + N-API 自动化测试链路，核心入口如下：

- 核心执行脚本：`scripts/ohos/run_split_tests_arkvm.sh`
- 测试拆分脚本：`scripts/ohos/split_ohos_tests.sh`

## 两种执行场景

### 1. 本地开发主机

本地开发主机默认通过 Docker 执行：

```bash
export ARK_HOST_TOOLS_DIR=/Users/ranger/Desktop/x64_linux_static
just unit-test-docker
```

等价命令：

```bash
export ARK_HOST_TOOLS_DIR=/Users/ranger/Desktop/x64_linux_static
export ARKVM_USE_DOCKER=1
bash ./scripts/ohos/run_split_tests_arkvm.sh
```

### 2. GitHub Actions

GitHub Actions runner 本身就是 Linux x64，因此直接在 host 环境执行：

- 先通过 `harmony-contrib/arkts-vm@v1.0.0` 初始化 ArkVM
- 然后执行 `scripts/ohos/run_split_tests_arkvm.sh`
- 显式设置 `ARKVM_USE_DOCKER=0`

## 约定

- `ARK_HOST_TOOLS_DIR` 指向 Linux x64 的 ArkJS N-API host bundle
- 动态库加载基于 `ark_js_napi_cli` + `requireNapiPreview(...)`
- `example` / `compact` 会以 `arkvm-test` feature 构建，用于把 `target_env = "ohos"` 相关代码编进宿主测试产物
- 实际执行依赖 `libace_napi.so`，由 host bundle 提供

## 保留这些脚本的原因

- `run_split_tests_arkvm.sh`：统一入口，按环境切换 Docker / host 执行模式
- `split_ohos_tests.sh`：把 Harmony 示例中的 Hypium 风格用例拆成可独立执行的 ArkTS suite
- `arkvm_build_host_libs.sh`：构建测试需要的本地 `.so`
- `arkvm_compile_abc_in_docker.sh`：负责 `.abc` 编译
- `arkvm_run_suites_in_docker.sh`：负责 suite 执行与结果汇总
