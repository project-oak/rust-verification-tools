#!/bin/bash

set -e

export RUSTFLAGS="-Clto -Cembed-bitcode=yes --emit=llvm-bc $RUSTFLAGS"
export RUSTFLAGS="--cfg=verify $RUSTFLAGS"
export RUSTFLAGS="-Warithmetic-overflow -Coverflow-checks=yes $RUSTFLAGS"
# export RUSTFLAGS="-Zpanic_abort_tests $RUSTFLAGS"
export RUSTFLAGS="-Cpanic=abort $RUSTFLAGS"

# optional for this simple example
export RUSTFLAGS="-Copt-level=1 $RUSTFLAGS"
export RUSTFLAGS="-Cno-vectorize-loops -Cno-vectorize-slp $RUSTFLAGS"
export RUSTFLAGS="-Ctarget-feature=-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2 $RUSTFLAGS"

cargo clean
cargo build --features=verifier-klee

# verify using KLEE
rm -rf kleeout
klee --libc=klee --silent-klee-assume --output-dir=kleeout --warnings-only-to-file target/debug/deps/try_klee*.bc

# view input value for first path
ktest-tool kleeout/test000001.ktest

# replay input values
KTEST_FILE=kleeout/test000001.ktest cargo run --features=verifier-klee
