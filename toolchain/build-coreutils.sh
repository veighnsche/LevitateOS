#!/bin/bash
# Build unmodified coreutils with c-gull sysroot

cd /home/vince/Projects/LevitateOS/toolchain/unmodified-coreutils

export RUSTFLAGS="-C panic=abort -C link-arg=-nostartfiles -C link-arg=-static -C link-arg=-Wl,--allow-multiple-definition -C link-arg=-L/home/vince/Projects/LevitateOS/toolchain/sysroot/lib"

# Note: Some utilities require libc functions not yet in c-gull:
# - ls: requires getpwuid, getgrgid
# - date: requires nl_langinfo
# Building only basic utilities that work:
cargo +nightly-2025-04-28 build --release \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort \
    --target x86_64-unknown-linux-gnu \
    --no-default-features \
    -p coreutils \
    --features "cat echo head mkdir pwd rm tail touch"
