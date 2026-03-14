# ArkVM 单测执行说明

当前仓库有两种单测执行场景：

- 本地开发主机：必须通过 Docker 执行
- GitHub Actions：runner 本身就是 Linux x64，直接在 host 环境执行，不再额外套 Docker

真实执行入口统一是：

- `scripts/ohos/run_split_tests_arkvm.sh`

## 场景一：本地开发主机

### 前置条件

#### 1. Docker 可用

```bash
docker version
```

#### 2. 准备 ArkVM host bundle

默认读取目录：

```bash
/Users/ranger/Desktop/x64_linux_static
```

如果你的 bundle 来自压缩包：

```bash
/Users/ranger/Desktop/arkvm_static_linux_x64.tar.gz
```

先解压：

```bash
cd /Users/ranger/Desktop
tar -xf arkvm_static_linux_x64.tar.gz
```

解压后目录内至少应包含：

- `es2abc`
- `ark_js_napi_cli`
- `libace_napi.so`
- `libets_interop_js_napi.so`

如果目录不是默认值，执行前设置：

```bash
export ARK_HOST_TOOLS_DIR=/path/to/x64_linux_static
```

### 执行命令

直接执行：

```bash
bash ./scripts/ohos/run_split_tests_arkvm.sh
```

或通过 `just`：

```bash
just unit-test-docker
```

本地默认会走 Docker 模式。

如果你想显式指定，也可以：

```bash
export ARKVM_USE_DOCKER=1
```

## 场景二：GitHub Actions

GitHub Actions 使用：

- `harmony-contrib/arkts-vm@v1.0.0` 初始化 ArkVM 环境
- Linux runner 直接执行 `scripts/ohos/run_split_tests_arkvm.sh`
- 不再构建额外测试执行 Docker

也就是说，GitHub Actions 与本地开发主机的差别只在“执行环境”：

- 本地：Docker
- GitHub Actions：host

如果你想在 Linux 主机上模拟 GitHub Actions 直跑模式，可以显式设置：

```bash
export ARKVM_USE_DOCKER=0
bash ./scripts/ohos/run_split_tests_arkvm.sh
```

## 常用环境变量

### `ARK_HOST_TOOLS_DIR`

指定 ArkVM host bundle 目录：

```bash
export ARK_HOST_TOOLS_DIR=/Users/ranger/Desktop/x64_linux_static
```

### `ARKVM_USE_DOCKER`

显式指定执行模式：

```bash
export ARKVM_USE_DOCKER=1  # 本地 Docker
export ARKVM_USE_DOCKER=0  # Linux host
```

### `KEEP_WORKDIR`

保留临时工作目录，便于排查：

```bash
export KEEP_WORKDIR=1
```

### `SKIP_BUILD_LIBS`

跳过重新构建本地测试依赖的 `.so`：

```bash
export SKIP_BUILD_LIBS=1
```

### `TEST_TIMEOUT_SEC`

单个 suite 的超时时间，默认 `90` 秒：

```bash
export TEST_TIMEOUT_SEC=120
```

## 脚本会做什么

无论 Docker 模式还是 host 模式，执行流程都一致：

1. 校验 ArkVM host bundle 是否完整
2. 构建测试需要的本地 `.so`
3. 在临时工作目录中复制 `test` 和 `third_party`
4. 基于 `test/ohos/source` 生成 split suites
5. 编译测试源码为 `.abc`
6. 通过 `ark_js_napi_cli` 逐个执行 suites
7. 汇总结果到 `results.tsv`

## 输出结果

结果文件默认位于：

```bash
.tmp_ohos_split_runner/results.tsv
```

设置：

```bash
export KEEP_WORKDIR=1
```

后，还可以继续查看：

- `.tmp_ohos_split_runner/logs/`
- `.tmp_ohos_split_runner/workspace/`

## 推荐排查命令

### 本地 Docker 模式

```bash
export ARK_HOST_TOOLS_DIR=/Users/ranger/Desktop/x64_linux_static
export KEEP_WORKDIR=1
export ARKVM_USE_DOCKER=1
bash ./scripts/ohos/run_split_tests_arkvm.sh
```

### Linux host 模式

```bash
export KEEP_WORKDIR=1
export ARKVM_USE_DOCKER=0
bash ./scripts/ohos/run_split_tests_arkvm.sh
```

### 查看失败 suite 日志

```bash
ls .tmp_ohos_split_runner/logs
cat .tmp_ohos_split_runner/logs/<suite_id>.log
```

## 常见问题

### 1. `Ark host bundle not found`

说明 `ARK_HOST_TOOLS_DIR` 指向的目录不存在，或者没有正确初始化 ArkVM bundle。

### 2. `Missing binary: es2abc` / `ark_js_napi_cli`

说明 bundle 不完整，或目录设置错误。

### 3. `Missing shared lib: libace_napi.so`

说明 bundle 缺少运行测试所需动态库。

### 4. 本地 Docker 权限错误

先确认：

```bash
docker version
```

如果你使用 OrbStack / Docker Desktop，先确认 daemon 已启动。
