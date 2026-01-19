#!/usr/bin/env -S just --justfile

_default:
    @just --list -u

init:
    cargo install typos-cli taplo-cli
    cargo install oxk --git https://github.com/ohos-rs/oxc-ark.git

ready:
    typos
    cargo fmt
    just check
    git status   

check:
    cargo fmt --check
    ohrs cargo clippy -- --workspace --target aarch64-unknown-linux-ohos
    ohrs cargo clippy -- --workspace --target armv7-unknown-linux-ohos
    ohrs cargo clippy -- --workspace --target x86_64-unknown-linux-ohos

fmt:
    cargo fmt
    taplo format
    oxk format "**/*.{ts,js,ets,json5}"

test:
    bash ./scripts/test.sh

zig-build:
    bash ./scripts/zig-build.sh