# Demo code

This directory contains demonstration code.

- `demos/simple/klee`: using KLEE directly to verify Rust code.
  See [docs/using-klee](../docs/using-klee.md) for description. 
- `demos/simple/annotations`: using the `verification-annotations` crate with
  `cargo-verify`.
  See [docs/using-annotations](../docs/using-annotations.md) for description. 

- `demos/simple/ffi`: using KLEE directly to verify Rust+C crates (that use the
  FFI).
  See [docs/using-ffi](../docs/using-ffi.md) for description. 
