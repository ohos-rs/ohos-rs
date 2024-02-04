# napi-rs-oh

> `napi-rs` HarmonyOS版本。整体代码来源于[napi-rs](https://github.com/napi-rs/napi-rs)，针对鸿蒙系统做了部分裁剪和整理

## 环境准备

- OpenHarmony SDK    
  配置环境变量为`OHOS_NDK_HOME`，进入之后目录结构应该包括 native ets js等目录

- Rust    
  MSRV 1.65.0

## 快速开始

1. 安装脚手架工具

```shell
cargo install ohrs
```

2. 初始化项目

```shell
ohrs init test
```

3. 构建产物

```shell
cd test

ohrs build
```

## 指南

目前所有API基本对齐napi，你可以在[example](https://github.com/ohos-rs/example)中看到在HarmonyOS工程中的应用。

## Packages

| Package                                                    | Version | Description                                                                |
|------------------------------------------------------------|---------| -------------------------------------------------------------------------- |
| [`@ohos-rs/crc32`](https://github.com/ohos-rs/crc32-ohos)  | 0.0.1   | Fastest `CRC32` implementation using `SIMD` |
| [`@ohos-rs/jieba`](https://github.com/ohos-rs/jieba-ohos)  | 0.0.1   | [`jieba-rs`](https://github.com/messense/jieba-rs) binding |


## 社区

你可以在 [这里](https://github.com/ohos-rs/example/issues) 提交相关问题、建议和需求场景。

## F&Q

1. 为什么Rust构建产物体积比官方的CMAKE构建体积大？    
Rust本身的产物体积会比C++大很多。即使使用了各种优化手段因为零抽象成本等各种原因，本身构建的代码体积就会比C++大。    
   - 如果对于体积较为敏感的话，可以参考[min-sized-rust](https://github.com/johnthagen/min-sized-rust)进行优化。
   - 另外可以使用NDK提供的strip工具进行优化。工具路径：`${OHOS_NDK_HOME}/native/llvm/bin/llvm-strip`


2. 为什么 `Option<T>` 参数会报错？    
目前OpenHarmony NDK中的 `napi_typeof` 方法实现有点问题，对于可选参数无法默认处理成 undefined 值。已向团队提相关问题，待修复即可。


3. 为什么 `Buffer` 在 Native 和 ArkTS 之间无法直接传递？    
ArkTS侧的 buffer实现跟Native侧的 Buffer实现不一致，导致在跨语言传递的时候出现问题。已向官方提相关问题，待修复即可。

## TODO
- [ ] CI支持
目前鸿蒙开放对于linux下通过NDK构建的能力支持太弱，等到API10放开后支持

- [ ] 鸿蒙官方底层能力包装
如hilog，vulkan，OpenGL等能力
