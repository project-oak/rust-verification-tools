#!/bin/bash

set -e

cargo clean

# this should detect multiple errors
cargo-verify --tests --verbose | tee out || true

# check for all expected errors
grep -q -F "test assert_eq_should_fail ... ASSERT_FAILED" out
grep -q -F "test bounds_should_fail ... OUT_OF_BOUNDS" out
grep -q -F "test overflow_should_fail ... OVERFLOW" out
grep -q -F "test panic_should_fail ... PANIC" out
grep -q -F "test prop_assert_should_fail ... ASSERT_FAILED" out
grep -q -F "test std_assert_should_fail ... ASSERT_FAILED" out
grep -q -F "test unwrap_should_fail ... PANIC" out

echo "Test detected all errors"
