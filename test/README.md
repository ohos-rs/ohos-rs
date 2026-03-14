# 本地通过 Docker 执行单测

本文档说明如何在本地通过 Docker 执行当前仓库的 ArkVM host 单测。

## 适用范围

当前这套流程主要覆盖 `test/ohos/source` 下的 split suites。
真实执行入口是：

- `scripts/ohos/run_split_tests_arkvm.sh`

也可以直接通过 `just` 调用：

- `just unit-test-docker`

## 前置条件

### 1. Docker 可用

先确认本机 Docker 能正常工作：

```bash
docker version
```

### 2. 准备 ArkVM host bundle

脚本默认会从下面的目录读取 ArkVM host 工具和动态库：

```bash
/Users/ranger/Desktop/x64_linux_static
```

如果你的 bundle 来自压缩包，例如：

```bash
/Users/ranger/Desktop/arkvm_static_linux_x64.tar.gz
```

先解压：

```bash
cd /Users/ranger/Desktop
tar -xf arkvm_static_linux_x64.tar.gz
```

解压后应至少包含这些文件：

- `es2abc`
- `ark_js_napi_cli`
- `libace_napi.so`
- `libets_interop_js_napi.so`

如果解压目录不是默认路径，执行前显式指定：

```bash
export ARK_HOST_TOOLS_DIR=/path/to/x64_linux_static
```

## 执行方式

### 方式一：直接执行脚本

```bash
bash ./scripts/ohos/run_split_tests_arkvm.sh
```

### 方式二：通过 just

```bash
just unit-test-docker
```

## 常用环境变量

### `ARK_HOST_TOOLS_DIR`

指定 ArkVM host bundle 目录：

```bash
export ARK_HOST_TOOLS_DIR=/Users/ranger/Desktop/x64_linux_static
```

### `KEEP_WORKDIR`

保留临时工作目录，便于排查：

```bash
export KEEP_WORKDIR=1
```

默认情况下，脚本执行完成后会清理工作目录。

### `SKIP_BUILD_LIBS`

跳过重新构建本地测试依赖的 `.so`：

```bash
export SKIP_BUILD_LIBS=1
```

仅当你确认 `target/arkvm-host/release` 下产物已经是最新时再使用。

### `TEST_TIMEOUT_SEC`

单个 suite 的超时时间，默认 `90` 秒：

```bash
export TEST_TIMEOUT_SEC=120
```

## 运行过程中会做什么

脚本会依次执行下面几步：

1. 校验 ArkVM host bundle 是否完整
2. 通过 Docker 构建测试需要的本地 `.so`
3. 在临时工作目录中复制 `test` 和 `third_party` 资源
4. 基于 `test/ohos/source` 生成 split suites
5. 在 Docker 内把测试源码编译成 `.abc`
6. 通过 `ark_js_napi_cli` 逐个执行 suite 并汇总结果

## 输出结果

默认结果文件位于：

```bash
.tmp_ohos_split_runner/results.tsv
```

如果设置了：

```bash
export KEEP_WORKDIR=1
```

还可以继续查看：

- `.tmp_ohos_split_runner/logs/`
- `.tmp_ohos_split_runner/workspace/`

## 推荐的本地排查命令

### 保留工作目录执行

```bash
export ARK_HOST_TOOLS_DIR=/Users/ranger/Desktop/x64_linux_static
export KEEP_WORKDIR=1
bash ./scripts/ohos/run_split_tests_arkvm.sh
```

### 查看失败 suite 日志

```bash
ls .tmp_ohos_split_runner/logs
cat .tmp_ohos_split_runner/logs/<suite_id>.log
```

## 常见问题

### 1. `Ark host bundle not found`

说明 `ARK_HOST_TOOLS_DIR` 指向的目录不存在，或者没有解压 ArkVM bundle。

### 2. `Missing binary: es2abc` / `ark_js_napi_cli`

说明 bundle 不完整，或目录设置错误。

### 3. `Missing shared lib: libace_napi.so`

说明 bundle 缺少运行测试所需动态库。

### 4. Docker 权限错误

先确认下面命令可正常返回：

```bash
docker version
```

如果你使用 OrbStack / Docker Desktop，先确认 daemon 已启动。
