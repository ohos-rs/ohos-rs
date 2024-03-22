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

### windows.Dockerfile

We now support Docker images for Windows, with the image size exceeding 10GB. However, this image is only intended for simple testing purposes on the Windows platform. For real CI/CD environments, we still recommend using Linux images.

## User Docker Image

Based with basic docker image. It can use to build package with CI/CD.

### ci.Dockerfile

CI example code,based with `debian.Dockerfile`

## config

rsproxy config