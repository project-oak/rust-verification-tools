---
layout: page
title: Documentation
permalink: /documentation/
---

- [Installation (using Docker)]({{site.baseurl}}{% link installation.md %})
  - [Crux-MIR]({{site.baseurl}}{% link install-crux.md %})

- Usage (using our tools)

  - [propverify]({{site.baseurl}}{% link using-propverify.md %}): a simple example to test
    `propverify` with.

  We also recommend reading
  [the proptest book](https://altsysrq.github.io/proptest-book/intro.html)
  that thoroughly explains and documents the `proptest` API that `propverify` is based on.

- How our tools work (in case you are creating your own tools)

  - [verification-annotations]({{site.baseurl}}{% link using-annotations.md %}): how to use the
    `verification-annotations` crate directly.
    Mostly interesting if you want to know how `propverify` works.

  - [using KLEE]({{site.baseurl}}{% link using-klee.md %}): how to use KLEE directly.
    Interesting if you want to know how `cargo-verify` works
    or if you are working with another LLVM-based verification tool.

  - [using FFI]({{site.baseurl}}{% link using-ffi.md %}): how to verify crates that use the
    foreign function interface (ffi) to call C code.
