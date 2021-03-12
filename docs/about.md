---
layout: page
title: About
---

RVT is a collection of tools/libraries to support both static
and dynamic verification of Rust programs.

We see static verification (formal verification) and dynamic verification
(testing) as two parts of the same activity and so these tools can be used for
either form of verification.

- Dynamic verification using the
  [proptest]
  fuzzing/property testing library.

- Static verification using the
  [KLEE]
  symbolic execution engine.

We aim to add other backends in the near future.

In addition, we write articles about how the tools we wrote work
(and you can read [the source][RVT git repo])
in case you are porting a verification tool for use with Rust.
(In particular, we describe how to generate LLVM bitcode files that can
be used with LLVM-based verification tools.)

## Articles


- [Installation (using Docker)][RVT installation]
  - If you want to use Crux-MIR, see these [alternative installation instructions][Install Crux]

- Usage (using our tools)

  - [propverify][Using PropVerify]: a simple example to test
    `propverify` with.

  We also recommend reading
  [the proptest book][PropTest book]
  that thoroughly explains and documents the `proptest` API that `propverify` is based on.

- How our tools work (in case you are creating your own tools)

  - [verification-annotations][Using verification-annotations]: how to use the
    `verification-annotations` crate directly.
    Mostly interesting if you want to know how `propverify` works.

  - [using KLEE][Using KLEE]: how to use KLEE directly.
    Interesting if you want to know how `cargo-verify` works
    or if you are working with another LLVM-based verification tool.

  - [using FFI][Using FFI]: how to verify crates that use the
    foreign function interface (ffi) to call C code.

## Usage

TL;DR

1. Install the dockerfile (see [instructions][RVT installation]).

    ``` shell
    git clone https://github.com/project-oak/rust-verification-tools.git
    cd rust-verification-tools
    docker/build
    ```

2. Fuzz some examples with proptest

   ```
   cd compatibility-test
   cargo test
   cd ..
   ```

   (You can also use
   `cargo-verify --backend=proptest --verbose compatibility-test`.)

   One test should fail – this is correct behaviour.

3. Verify some examples with propverify

   `cargo-verify --tests verification-annotations`

   `cargo-verify --tests compatibility-test`

   No tests should fail.

4. Read [the propverify intro][Using PropVerify] for an example
   of fuzzing with `proptest` and verifying with `propverify`.

5. Read [the proptest book][PropTest book]

6. Read the source code for the [compatibility test suite][compatibility-test].

   (Many of these examples are taken from or based on examples in
   [the proptest book][PropTest book].)



## Installation

Follow the [installation instructions][RVT installation].


## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE][LICENSE-APACHE] or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT][LICENSE-MIT] or
  http://opensource.org/licenses/MIT)

at your option.


## Acknowledgements

The `propverify` crate is heavily based on the design and API of the wonderful
[proptest](https://github.com/AltSysrq/proptest)
property/fuzz-testing library.
The implementation also borrows techniques, tricks and code
from the implementation – you can learn a lot about how to write
an embedded DSL from reading the proptest code.

In turn, `proptest` was influenced by
the [Rust port of QuickCheck](https://github.com/burntsushi/quickcheck)
and
the [Hypothesis](https://hypothesis.works/) fuzzing/property testing library for Python.
(`proptest` also acknowledges `regex_generate` – but we have not yet implemented
regex strategies for this library.)


## Known limitations

This is not an officially supported Google product;
this is an early release of a research project
to enable experiments, feedback and contributions.
It is probably not useful to use on real projects at this stage
and it may change significantly in the future.

Our current goal is to make `propverify` as compatible with
`proptest` as possible but we are not there yet.
The most obvious features that are not even implemented are
support for
using regular expressions for string strategies,
the `Arbitrary` trait,
`proptest-derive`.

We would like the `propverify` library and the `cargo-verify` script
to work with as many Rust verification tools as possible
and we welcome pull requests to add support.
We expect that this will require design/interface changes.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as
above, without any
additional terms or conditions.

See [the contribution instructions][CONTRIBUTING] for further details.

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
[SMACK]:                          https://smackers.github.io/
[SV-COMP]:                        https://sv-comp.sosy-lab.org/2020/rules.php
[std::env::args source code]:     https://github.com/rust-lang/rust/blob/master/library/std/src/sys/unix/args.rs

[RVT git repo]:                   {{site.gitrepo}}/
[cargo-verify source]:            {{site.gitrepo}}blob/main/cargo-verify/
[compatibility-test]:             {{site.gitrepo}}blob/main/compatibility-test/src
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

[RVT installation]:               {{site.baseurl}}{% link installation.md %}

