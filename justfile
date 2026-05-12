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
    just third-party-build
    bash ./scripts/test.sh

unit-test:
    bash ./scripts/ci/run_ets_napi_host_tests.sh

unit-test-arkvm:
    bash ./scripts/arkvm/run_tests.sh

zig-build:
    bash ./scripts/zig-build.sh

third-party-build:
    ohrs build -p ohos-buffer-host
    ohrs build -p ohos-worker-host