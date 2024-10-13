#!/usr/bin/env -S just --justfile

_default:
    @just --list -u

init:
    cargo binstall typos-cli taplo-cli -y

ready:
    typos
    cargo fmt
    just check
    just lint
    git status

lint:
    cargo lint -- --deny warnings

check:
    cargo ck

fmt:
    cargo fmt
    taplo format
    npx prettier --write '**/*.(ts|js|ets)' --trailing-comma=none

test:
    bash ./scripts/test.sh
