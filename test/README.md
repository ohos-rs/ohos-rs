# ArkVM 单测执行说明

ArkTS + N-API 单测只支持 Linux host 环境执行。CI runner 本身就是 Linux，因此测试入口不再支持 Docker，也不负责安装环境。

真实执行入口：

- `scripts/arkvm/run_tests.sh`

## 前置条件

### 1. ArkVM host bundle

执行前设置：

```bash
export ARK_HOST_TOOLS_DIR=/path/to/x64_linux_static
```

目录内至少应包含：

- `es2abc`
- `ark_js_napi_cli`
- `libace_napi.so`
- `libets_interop_js_napi.so`

### 2. Host 测试动态库

`run_tests.sh` 只编译 ArkTS/TS/JS 测试源码并执行 suite，不构建 native `.so`。

执行测试前先构建：

```bash
bash ./scripts/arkvm/build_host_libs.sh
```

默认输出目录：

```bash
target/arkvm-host/release
```

如需使用其他目录：

```bash
export TARGET_RELEASE_DIR=/path/to/release
```

## 执行命令

```bash
bash ./scripts/arkvm/build_host_libs.sh
bash ./scripts/arkvm/run_tests.sh
```

或通过 `just`：

```bash
just unit-test
```

## 常用环境变量

### `ARK_HOST_TOOLS_DIR`

指定 ArkVM host bundle 目录：

```bash
export ARK_HOST_TOOLS_DIR=/path/to/x64_linux_static
```

### `TARGET_RELEASE_DIR`

指定测试 `.so` 所在目录：

```bash
export TARGET_RELEASE_DIR=/path/to/release
```

### `KEEP_WORKDIR`

保留临时工作目录，便于排查：

```bash
export KEEP_WORKDIR=1
```

### `TEST_TIMEOUT_SEC`

单个 suite 的超时时间，默认 `90` 秒：

```bash
export TEST_TIMEOUT_SEC=120
```

## `run_tests.sh` 会做什么

1. 校验 ArkVM host bundle 和测试 `.so`
2. 在临时工作目录中复制 `test` 和 `third_party`
3. 基于 `test/source` 生成 split suites
4. 编译测试源码为 `.abc`
5. 通过 `ark_js_napi_cli` 逐个执行 suites
6. 汇总结果到 `results.tsv`

## 输出结果

结果文件默认位于：

```bash
.tmp_arkvm_runner/results.tsv
```

设置：

```bash
export KEEP_WORKDIR=1
```

后，还可以继续查看：

- `.tmp_arkvm_runner/logs/`
- `.tmp_arkvm_runner/workspace/`

## 常见问题

### 1. `ARK_HOST_TOOLS_DIR or ARK_HOST_BUNDLE_DIR is required`

说明没有指定 ArkVM host bundle。

### 2. `Missing binary: es2abc` / `ark_js_napi_cli`

说明 bundle 不完整，或目录设置错误。

### 3. `Missing host test library`

说明还没有构建测试依赖的 native `.so`，先执行：

```bash
bash ./scripts/arkvm/build_host_libs.sh
```
