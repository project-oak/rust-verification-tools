---
layout: page
title: Tools and libraries
permalink: /tools/
---

## Tools

- `cargo-verify`: a tool for compiling a crate and
  either verifying main/tests or for fuzzing main/tests.
  (Use the `--backend` flag to select which.)

- `rvt-patch-llvm`: a tool for preprocessing LLVM bitfiles before verification.
  (Used by `cargo-verify`.)

- `rust2calltree`: a tool for fixing (demangling) function names in
  kcachegrind profile files.


## Libraries

- `verification-annotations` crate: an FFI layer for creating symbolic values in
  [KLEE](http://klee.github.io/)

- `propverify` crate:
  an implementation of the [proptest](https://github.com/AltSysrq/proptest)
  library for use with static verification tools.

- `compatibility-test` test crate:
  test programs that can be verified either using the original `proptest`
  library or using `propverify`.
  Used to check that proptest and propverify are compatible with each other.


