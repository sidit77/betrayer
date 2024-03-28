#!/usr/bin/env just --justfile

set windows-shell := ["powershell.exe", "-c"]

release:
  cargo build --release    

lint:
  cargo clippy

fmt:
  cargo +nightly fmt

check-windows:
    cargo check --all-features --target x86_64-pc-windows-msvc

check-linux:
    cargo check --all-features --target x86_64-unknown-linux-gnu

check-macos:
    cargo check --all-features --target x86_64-apple-darwin

check: check-windows check-linux check-macos