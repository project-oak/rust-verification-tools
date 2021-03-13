---
layout: page
title: Tools and libraries
permalink: /tools/
---

## Tools

- `cargo-verify`: a tool for compiling a crate and
  either verifying main/tests or for fuzzing main/tests.
  (Use the `--backend` flag to select which.)

  `cargo-verify` uses similar command-line flags to the standard `cargo-test` tool.
  The `--script=PATH` flag generates a list of all the commands executed
  by `cargo-verify`.

  The source code is [here][cargo-verify source].

- `rvt-patch-llvm`: a tool for preprocessing LLVM bitfiles before verification.
  (Used by `cargo-verify`.)

  This fixes problems in LLVM bitfiles that are designed to be used for
  compilation or execution including

  - `--features` causes all processor feature test functions to return false.
    This is useful for disabling hand-vectorized code.

    See [Using FFI] for details.

  - `--initializers` causes `main` to call all initializers before it runs.
    This is useful for programs that use `std::env::args()` to access the
    command line arguments.

    See [Using ARGV] for details.

  - `--seahorn` fixes various problems that affect the [SeaHorn] backend.

  The source code is [here][rvt-patch-llvm source].

- `rust2calltree`: a tool for fixing (demangling) function names in
  kcachegrind profile files.

  See [Profiling Rust] for usage.

  The source code is [here][rust2calltree source].


## Libraries

- [`verification-annotations` crate][verification-annotations source]: an FFI layer for creating symbolic values in
  [KLEE], [Crux-MIR] or [SeaHorn].

  See [Using verification-annotations] for details.

- [`propverify` crate][propverify source]:
  an implementation of the [proptest](https://github.com/AltSysrq/proptest)
  library for use with static verification tools.

  See [Using PropVerify] for usage.


- [`compatibility-test` test crate][compatibility-test]:
  test programs that can be verified either using the original `proptest`
  library or using `propverify`.
  Used to check that proptest and propverify are compatible with each other.

  Partly based on examples in the [PropTest book].


[CC-rs crate]:                    https://github.com/alexcrichton/cc-rs/
[Cargo build scripts]:            https://doc.rust-lang.org/cargo/reference/build-scripts.html
[Clang]:                          https://clang.llvm.org/
[Crux-MIR]:                       https://github.com/GaloisInc/mir-verifier/
[Docker]:                         https://www.docker.com/
[GraalVM and Rust]:               https://michaelbh.com/blog/graalvm-and-rust-1/
[Hypothesis]:                     https://hypothesis.works/
[KLEE]:                           https://klee.github.io/
[Linux driver verification]:      http://linuxtesting.org/ldv/
[LLVM]:                           https://llvm.org/
[MIR blog post]:                  https://blog.rust-lang.org/2016/04/19/MIR.html
[PropTest book]:                  https://altsysrq.github.io/proptest-book/intro.html
[PropTest]:                       https://github.com/AltSysrq/proptest/
[Rust benchmarks]:                https://github.com/soarlab/rust-benchmarks/
[Rust port of QuickCheck]:        https://github.com/burntsushi/quickcheck/
[Rust's runtime]:                 https://blog.mgattozzi.dev/rusts-runtime/
[SeaHorn]:                        https://seahorn.github.io/
[SMACK]:                          https://smackers.github.io/
[SV-COMP]:                        https://sv-comp.sosy-lab.org/2020/rules.php
[std::env::args source code]:     https://github.com/rust-lang/rust/blob/master/library/std/src/sys/unix/args.rs

[HATRA 2020]:                     https://alastairreid.github.io/papers/HATRA_20/

[RVT git repo]:                   {{site.gitrepo}}/
[cargo-verify source]:            {{site.gitrepo}}blob/main/cargo-verify/
[verification-annotations source]: {{site.gitrepo}}blob/main/verification-annotations/
[rust2calltree source]:           {{site.gitrepo}}blob/main/rust2calltree/
[rvt-patch-llvm source]:          {{site.gitrepo}}blob/main/rvt-patch-llvm/
[compatibility-test]:             {{site.gitrepo}}blob/main/compatibility-test/src
[propverify source]:              {{site.gitrepo}}blob/main/propverify/
[demos/simple/ffi directory]:     {{site.gitrepo}}blob/main/demos/simple/ffi/
[CONTRIBUTING]:                   {{site.gitrepo}}blob/main/CONTRIBUTING.md
[LICENSE-APACHE]:                 {{site.gitrepo}}blob/main/LICENSE-APACHE
[LICENSE-MIT]:                    {{site.gitrepo}}blob/main/LICENSE-MIT

[Using KLEE]:                     {{site.baseurl}}{% post_url 2020-09-01-using-klee %}
[Using verification-annotations]: {{site.baseurl}}{% post_url 2020-09-02-using-annotations %}
[Using PropVerify]:               {{site.baseurl}}{% post_url 2020-09-03-using-propverify %}
[Install Crux]:                   {{site.baseurl}}{% post_url 2020-09-07-install-crux %}
[Using ARGV]:                     {{site.baseurl}}{% post_url 2020-09-09-using-argv %}
[Using FFI]:                      {{site.baseurl}}{% post_url 2020-12-11-using-ffi %}
[Profiling Rust]:                 {{site.baseurl}}{% post_url 2021-03-12-profiling-rust %}

