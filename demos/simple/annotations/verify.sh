#!/bin/bash

set -e

cargo clean

# verify using KLEE
# this should detect an error
cargo-verify --tests --verbose | tee out1 || true
grep -q -F "test t1 ... ASSERT_FAILED" out1

# replay input values
cargo-verify --tests --replay | tee out2 || true
grep -q -F "Test values: a = 1000, b = 1000" out2
