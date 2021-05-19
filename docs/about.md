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


## Usage

Here's the basics of getting started with our tools.

1. Build a docker image

   Rust verification is relatively new and we are trying to use multiple
   verification tools so, at least for now, these libraries have many complex dependencies
   that are best handled by using [Docker].

    ``` shell
    git clone https://github.com/project-oak/rust-verification-tools.git
    cd rust-verification-tools
    docker/build
    ```

   This will take about 15-20 minutes to build the Docker images.
   The resulting docker image can be run by executing `docker/run`
   which executes a bash shell using the current user in the current directory.

   If building on OSX, you should increase the container memory limit before
   running `docker/build` using the slider in the Configure:Resources menu.
   We tested with 8GB but 4GB is probably enough.

   If you are unable to use Docker, the best approach is to manually execute
   the commands in the Dockerfiles invoked at the end of the [docker/build script][docker-build].


2. Open a docker shell

   ``` shell
   docker/run
   ```

   All subsequent instructions assume that you are running inside docker.
   We normally run docker in one terminal and run an editor, git, etc.
   in a normal (non-docker) terminal.

   If you have changed RVT since building your docker image
   (e.g., you edited the RVT tools/libraries or did a `git pull`),
   you should run `docker/init` each time you start a new
   docker shell to make sure that the tools and libraries have been
   rebuilt.

2. Fuzz some examples with proptest

   ```
   cd compatibility-test
   cargo test
   cd ..
   ```

   (You can also use
   `cargo-verify --backend=proptest --verbose`.)

   One test should fail – this is correct behaviour.

3. Verify some examples with propverify

   `cd verification-annotations; cargo-verify --tests`

   or

   `cd compatibility-test; cargo-verify --tests`

   No tests should fail.

4. Read [the propverify intro][Using PropVerify] for an example
   of fuzzing with `proptest` and verifying with `propverify`.

5. Read [the proptest book][PropTest book]

6. Read the source code for the [compatibility test suite][compatibility-test].

   (Many of these examples are taken from or based on examples in
   [the proptest book][PropTest book].)


## Articles: Our goals and plans

- We wrote a paper about our vision for Rust verification "[Towards making formal methods normal: meeting developers where they are][HATRA 2020]"
  about tool usability,
  the vision of building on developers existing comfort and familiarity with testing and fuzzing,
  and the challenges of getting adoption of formal verification in large organizations.


## Articles: Using our tools and libraries

- [Using the `propverify` library to verify Rust programs][Using PropVerify]

  Demonstrates the idea of writing a single verification harness that
  can be used for testing (using the [PropTest] structure-aware fuzzing library)
  or for verification (using the [KLEE] symbolic execution engine).

  Also shows the basics of using our `cargo-verify` adaptation of `cargo-test`.

  We also recommend reading
  [the proptest book][PropTest book]
  that thoroughly explains and documents the `proptest` API that `propverify` is based on.

- [Profiling Rust verification][Profiling Rust]

  Formal verification tools push up against Leino's
  "decidability ceiling": taking the risk of trying
  to solve undecidable problems in order to create more powerful tools.
  The cost of this is that sometimes the verifier will "blow up"
  on some part of your program.

  This article is about finding the problem code so that you
  can try to fix it.

## Articles: Under the hood

- [Using the `verification-annotations` crate][Using verification-annotations]

  Demonstrates the verification API underlying `propverify`.

- [Using KLEE][Using KLEE]

  Explains how to compile Rust programs to generate an LLVM file
  that can be used for verification and then how to use KLEE
  with that file.

  This is a (slightly simplified) explanation of what `cargo-verify`
  does internally.
  (To really see how `cargo-verify` works, we recommend using the
  `--script=PATH` flag when using `cargo-verify`.)

- [Crux-Mir installation][Install Crux]

  Some slightly out of date instructions on using the [Crux-MIR]
  Rust verifier with our libraries.

- [Using command-line arguments ('argv')][Using ARGV]

  We have been working on identifying features of realistic
  Rust programs that prevent you from verifying them.
  A surprising blocker was that we could not pass
  command line arguments to Rust when verifying them
  and so we could not do any meaningful verification of
  most Rust programs.

  This article delves into how command line arguments
  (i.e., `std::env::args()`) work in Rust and
  how our `rvt-patch-llvm` preprocessor patches the
  LLVM file to make initializers work in verifiers.

- [Using Rust's foreign function interface][Using FFI]

  Another common feature of Rust crates is that they
  are partly written in C and use Alex Crichton's
  [CC-rs crate] to compile the C code.
  This was a major blocker because we need LLVM code for
  the entire application, not just the Rust
  parts of the application.

  This article describes how to arrange that the C code
  is compiled to LLVM instead of x86/Arm machine code.

- [Verifying hand-vectorized Rust code][Hand vectorized Rust]

  Verification tools don't support every intrinsic function for every
  architecture: so they may reject hand-vectorized code.

  This article describes how to bypass hand vectorized code in portable
  libraries that provide an unoptimized fallback path.


## License

Our RVT tools and libraries are licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE][LICENSE-APACHE] or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT][LICENSE-MIT] or
  http://opensource.org/licenses/MIT)

at your option.

Our tools invoke [KLEE], [PropTest], [Crux-MIR], [SeaHorn], etc.
which generally have flexible open-source licenses as well.


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
[SeaHorn]:                        https://seahorn.github.io/
[SMACK]:                          https://smackers.github.io/
[SV-COMP]:                        https://sv-comp.sosy-lab.org/2020/rules.php
[std::env::args source code]:     https://github.com/rust-lang/rust/blob/master/library/std/src/sys/unix/args.rs

[HATRA 2020]:                     https://alastairreid.github.io/papers/HATRA_20/

[RVT git repo]:                   {{site.gitrepo}}/
[cargo-verify source]:            {{site.gitrepo}}blob/main/cargo-verify/
[compatibility-test]:             {{site.gitrepo}}blob/main/compatibility-test/src
[demos/simple/ffi directory]:     {{site.gitrepo}}blob/main/demos/simple/ffi/
[CONTRIBUTING]:                   {{site.gitrepo}}blob/main/CONTRIBUTING.md
[LICENSE-APACHE]:                 {{site.gitrepo}}blob/main/LICENSE-APACHE
[LICENSE-MIT]:                    {{site.gitrepo}}blob/main/LICENSE-MIT
[docker-build]:                   {{site.gitrepo}}blob/main/docker/build

[Using KLEE]:                     {{site.baseurl}}{% post_url 2020-09-01-using-klee %}
[Using verification-annotations]: {{site.baseurl}}{% post_url 2020-09-02-using-annotations %}
[Using PropVerify]:               {{site.baseurl}}{% post_url 2020-09-03-using-propverify %}
[Install Crux]:                   {{site.baseurl}}{% post_url 2020-09-07-install-crux %}
[Using ARGV]:                     {{site.baseurl}}{% post_url 2020-09-09-using-argv %}
[Using FFI]:                      {{site.baseurl}}{% post_url 2020-12-11-using-ffi %}
[Profiling Rust]:                 {{site.baseurl}}{% post_url 2021-03-12-profiling-rust %}
[Hand vectorized Rust]:           {{site.baseurl}}{% post_url 2021-03-20-verifying-vectorized-code %}

