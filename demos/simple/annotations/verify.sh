#!/bin/bash

set -e

cargo clean

# verify using KLEE
# this should detect an error
( cargo-verify . --tests --verbose > out1 || true )
cat out1
grep -q "test t1 ... .*ERROR" out1

# replay input values
( cargo-verify . --tests --replay > out2 || true )
cat out2
grep -q "Test values: a = 1000, b = 1000" out2
