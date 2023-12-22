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

## F&Q

1. 为什么Rust构建产物体积比官方的CMAKE构建体积大？    
因为官方cmake将STL作为了一个libc++_stared单独构建，因此目标产物较小，Rust没有做这一步。最终比较下来Rust的构建体积应该是小于cmake的

2. 

## TODO
- [ ] CI支持
目前鸿蒙开放对于linux下通过NDK构建的能力支持太弱，等到API10放开后支持

- [ ] 鸿蒙官方底层能力包装
如hilog，vulkan，OpenGL等能力
