#!/usr/bin/env -S just --justfile

_default:
    @just --list -u

init:
    cargo binstall typos-cli taplo-cli -y

ready:
    git diff --exit-code --quiet
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
