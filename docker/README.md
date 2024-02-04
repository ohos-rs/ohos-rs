# CI/CD

## Basic Docker Image
Provide some basic env.
- NDK HOME
- Rust toolchain with [rsproxy](https://rsproxy.cn/)

### debian.Dockerfile

Based with `debian:stable`

## User Docker Image

Based with basic docker image. It can use to build package with CI/CD.

### ci.Dockerfile
Based with `debian.Dockerfile`
