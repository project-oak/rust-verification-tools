#!/bin/bash

set -x
set -e

export RUSTFLAGS="-Clto -Cembed-bitcode=yes --emit=llvm-bc $RUSTFLAGS"
export RUSTFLAGS="--cfg=verify $RUSTFLAGS"
export RUSTFLAGS="-Warithmetic-overflow -Coverflow-checks=yes $RUSTFLAGS"
export RUSTFLAGS="-Zpanic_abort_tests -Cpanic=abort $RUSTFLAGS"

# optional for this simple example
# export RUSTFLAGS="-Copt-level=1 $RUSTFLAGS"
# export RUSTFLAGS="-Cno-vectorize-loops -Cno-vectorize-slp $RUSTFLAGS"
# export RUSTFLAGS="-Ctarget-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2 $RUSTFLAGS"

cargo clean
cargo build --features=verifier-seahorn

rvt-patch-llvm -o try_seahorn.patch.bc --seahorn -vv target/debug/deps/try_seahorn-*.bc

# Find the mangled main function
MAIN="$(llvm-nm-${LLVM_VERSION} --defined-only try_seahorn.patch.bc | grep main | cut -d ' ' -f3)"

# verify using SeaHorn
rm -rf seaout
sea yama -y ${SEAHORN_VERIFY_C_COMMON_DIR}/seahorn/sea_base.yaml bpf --temp-dir=seaout --entry="${MAIN}" try_seahorn.patch.bc

# To get a trace:
# sea yama -y ${SEAHORN_VERIFY_C_COMMON_DIR}/seahorn/sea_cex_base.yaml bpf --temp-dir=seaout --entry="${MAIN}" try_seahorn.patch.bc
