# napi-rs-oh

> `napi-rs` HarmonyOS版本。整体代码来源于[napi-rs](https://github.com/napi-rs/napi-rs)，针对鸿蒙系统做了部分裁剪和整理

目前仅用于本地测试可行性，后续将参考napi-rs做项目整体开发。

#### 环境准备

- OpenHarmony SDK    
  配置环境变量为`OHOS_NDK_HOME`，进入之后目录结构应该包括 native ets js等目录
- Rust    
  MSRV 1.65.0
#### 编译

```shell
# armv8a
cargo +nightly build --target aarch64-unknown-linux-ohos -Z build-std --release

# armv7a
cargo +nightly build --target armv7-unknown-linux-ohos -Z build-std --release

# x86_64
cargo +nightly build --target x86_64-unknown-linux-ohos -Z build-std --release
```

#### 功能
- [x] napi-rs整体运行
- [x] 裁剪不支持功能
- [ ] 脚手架支持
- [ ] CI/CD支持
