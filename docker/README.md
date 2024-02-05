# CI/CD

## Basic Docker Image
Provide some basic env.
- NDK HOME
- Rust toolchain with [rsproxy](https://rsproxy.cn/)

Basic build command with Apple M-series chip:

```shell
docker build --platform linux/x86_64 -t southorange/ohos-base:v4 -f debian.Dockerfile .
```

### debian.Dockerfile

Based with `debian:stable`

## User Docker Image

Based with basic docker image. It can use to build package with CI/CD.

### ci.Dockerfile

CI example code,based with `debian.Dockerfile`

## config

rsproxy config