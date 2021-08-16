#!/bin/bash

set -e

export RUSTFLAGS="--deny warnings --cfg=verify -Z always_encode_mir $RUSTFLAGS"

cargo clean
cargo build --features=verifier-mirai

# verify using MIRAI

# default|verify|library|paranoid
export MIRAI_FLAGS="--diag=paranoid"
touch src/main.rs
RUSTC_WRAPPER="mirai" cargo build --features=verifier-mirai
