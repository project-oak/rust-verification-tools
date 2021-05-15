---
layout: post
title: Rust/KLEE status update
---

![KLEE logo](https://klee.github.io/images/klee.svg){: style="float: left; width: 10%; padding: 1%"}
A lot of [our work on Rust formal verification][RVT git repo] is based on [LLVM] based tools
and, in particular, the [KLEE] symbolic execution tool that can be used
to find bugs and to generate high coverage testsuites.
We still have more work to do but it's a good time for a Rust/KLEE status update.

### Most things work

As part of our approach of ["Meeting developers where they are"][reid:hatra:2020],
our goal so far has been to figure out how to take an arbitrary Rust crate and turn it into a bitcode file that an
LLVM-based tool such as [KLEE] can use.

As a user of our tools and libraries, what you will mostly see is

- Our ["cargo-verify" script][cargo-verify source] can build many real-world crates and generate bitcode that KLEE can use.

  - "cargo-verify" behaves a lot like "cargo test".

  - (Most of) how we persuade cargo build to generate bitcode files for KLEE is
    [described here][Using KLEE].

- Our ["verification-annotations" crate][using verification-annotations] supports almost all of the functions in
  KLEE's intrinsics.
  In particular it provides
  - A Rust trait for creating symbolic values implemented for all the primitive types.
  - Functions for adding assumptions and for reporting errors.

  This API is also implemented for other verification tools such as
  [SeaHorn] and [Crux-Mir].

- Our ["prop-verify" crate][Using propverify] provides a Domain Specific Language (DSL) on top of
  "verification-annotations" for easily building complex symbolic values.
  The DSL is also implemented by the ["prop-test" fuzz-tester][PropTest]
  so you can [use your verification harnesses for fuzzing and your fuzzing harnesses for verification][Rust testing or verification].

- We can [use kcachegrind][Profiling Rust] to find verification bottlenecks in Rust code
  (using [a simple tool][rust2calltree] to demangle function names correctly).

- We have a Docker container for all the tools (because installing the full set of tools and their
  dependencies is quite complicated).


While the use of LLVM means that all the obvious Rust language features like
closures and memory allocation "just work", real programs depend on a bunch of
other "language features" that are provided by the compiler,
linker, package manager or popular libraries and we have been slowly
working away on supporting all of these.

- To get bitcode for the standard library, we build the Rust compiler and standard library ourselves with [just the right flags][rustc Dockerfile].

- Rust has enthusiastically switched to LLVM-11 (the latest LLVM) but not all verification tools support this yet.
  To generate the more widely supported LLVM-10, we use a Rust compiler from around August 2020 just before the
  Rust compiler switched to LLVM-11.

- To [support command line arguments (std::env::args())][Using argv],
  our preprocessor "rvt-patch-llvm" arranges that initializers are invoked at the start of main.
  (KLEE already had a similar behaviour built into it -- but for a different type of initializer so
  it didn't help.)

- To [support Rust's foreign function interface][Using FFI], our script "cargo-verify" arranges that
  we generate and link in bitcode for any C code in Rust crates.

- To [avoid the use of vector instructions][Hand vectorized Rust],

    1. We compile Rust code with auto-vectorization disabled.

    2. Our preprocessor "rvt-patch-llvm" modifies code used to dynamically detect
       the presence of vector support in the processor to say that SSE2, AVX2, etc. are not
       supported.

    3. We are in the process of revising the set of flags used to compile Rust's
       standard library. It seems that the "hashbrown" hashing library has some
       hand-vectorized vector code that statically tests whether the processor
       supports SSE2.
       Sadly, turning off SSE2 can cause the compiler to generate code for the x87 FPU 😕.


With all this in place, we are starting to be able to use KLEE
with interestingly large programs such as [uutils / CoreUtils][Rust coreutils]: a Rust
rewrite of coreutils that can more-or-less be used as a drop-in replacement
for the GNU originals.
This is a fun choice because one of KLEE's original demonstrations was finding bugs
in the GNU coreutils suite.
Whether the Rust version has any of the properties that make coreutils
a good choice for KLEE benchmarking remains to be seen.


## More still to do

There are a bunch of things that still need work though.

- Threads -- while it is ok to use thread-safe code that protects itself using locks,
  it is not ok to have more than one thread
  😕.

- Dynamic linking seems to be causing problems.

- Some crates have hand-written assembly language.
  In many cases, there is both an assembly version and a Rust version of the same
  code -- I hope to be able to persuade the crates to just use the Rust version.

- On the design side, our cargo-verify script combines building the bitcode file
  with running KLEE on the bitcode.
  This makes it behave like "cargo test" and seemed like a good design choice.
  when we could only tackle toy examples.
  But now that we are tackling larger examples with longer build times and
  much longer verification times, we are
  starting to think about separating this into two separate phases.

- The main trick for extracting LLVM bitcode for a program is to use link-time
  optimization (LTO). Unfortunately, LTO can be quite slow: we need to find
  a way to get LTO to do less optimization!


We have also been experimenting with how to use the rest of [KLEE's API](https://github.com/klee/klee/blob/master/include/klee/klee.h)
(see KLEE's documentation
[[#1](https://klee.github.io/docs/intrinsics/),
[#2](https://klee.github.io/tutorials/testing-function/)]).
In particular, we have experimental interfaces for 
testing whether an expression is symbolic,
a Rust trait for concretization of symbolic values,
and
a macro for merging paths within a block of code.



## Summary

So we have tools that can handle reasonably complex Rust applications,
and we are starting to use them to find inputs that trigger panics and other runtime exceptions.
There is more work to be done but it feels like our tools are getting close to being useful.

That said, there is a difference between a "useful" research quality tool and a
"usable" tool to use as part of your everyday development and I suspect that
our tools are still a bit rough for all but the most enthusiastic to tolerate.
In the meantime though, we would love to talk to other people working on using
[KLEE] or other [LLVM]-based tools with Rust:[^licenses]
How did you solve the problems you ran into?
What can you do that we can't do?
Can we combine our ideas?

[^licenses]:
    All of our code is MIT+Apache dual-licensed so you should be able to use the code or borrow code/ideas from it if you wish.


-----------------

[aho-corasick crate]:             https://crates.io/crates/aho-corasick/
[CC-rs crate]:                    https://github.com/alexcrichton/cc-rs/
[Cargo build scripts]:            https://doc.rust-lang.org/cargo/reference/build-scripts.html
[Clang]:                          https://clang.llvm.org/
[Crux-MIR]:                       https://github.com/GaloisInc/mir-verifier/
[Docker]:                         https://www.docker.com/
[GraalVM and Rust]:               https://michaelbh.com/blog/graalvm-and-rust-1/
[Hypothesis]:                     https://hypothesis.works/
[kcachegrind]:                    https://kcachegrind.github.io/html/Home.html
[KLEE]:                           https://klee.github.io/
[Linux driver verification]:      http://linuxtesting.org/ldv/
[LLVM]:                           https://llvm.org/
[MIR blog post]:                  https://blog.rust-lang.org/2016/04/19/MIR.html
[PropTest book]:                  https://altsysrq.github.io/proptest-book/intro.html
[PropTest]:                       https://github.com/AltSysrq/proptest/
[regex crate]:                    https://crates.io/crates/regex
[Rust benchmarks]:                https://github.com/soarlab/rust-benchmarks/
[Rust CoreUtils]:                 https://github.com/uutils/coreutils
[Rust port of QuickCheck]:        https://github.com/burntsushi/quickcheck/
[Rust's runtime]:                 https://blog.mgattozzi.dev/rusts-runtime/
[SeaHorn]:                        https://seahorn.github.io/
[SMACK]:                          https://smackers.github.io/
[SV-COMP]:                        https://sv-comp.sosy-lab.org/2020/rules.php
[std-detect]:                     https://docs.rs/std_detect/0.1.5/std_detect/
[std::std_detect::detect::check_for]: https://stdrs.dev/nightly/x86_64-pc-windows-gnu/std/std_detect/detect/fn.check_for.html
[std::env::args source code]:     https://github.com/rust-lang/rust/blob/master/library/std/src/sys/unix/args.rs

[RVT git repo]:                   {{site.gitrepo}}/
[cargo-verify source]:            {{site.gitrepo}}blob/main/cargo-verify/
[verification-annotations source]: {{site.gitrepo}}blob/main/verification-annotations/
[compatibility-test]:             {{site.gitrepo}}blob/main/compatibility-test/src
[demos/simple/ffi directory]:     {{site.gitrepo}}blob/main/demos/simple/ffi/
[CONTRIBUTING]:                   {{site.gitrepo}}blob/main/CONTRIBUTING.md
[LICENSE-APACHE]:                 {{site.gitrepo}}blob/main/LICENSE-APACHE
[LICENSE-MIT]:                    {{site.gitrepo}}blob/main/LICENSE-MIT
[regex bottleneck]:               {{site.gitrepo}}blob/main/demos/bottlenecks/regex/src/main.rs
[rustc Dockerfile]:               {{site.gitrepo}}blob/main/docker/rustc/Dockerfile
[rust2calltree]:                  {{site.gitrepo}}tree/main/rust2calltree
[rvt-patch-llvm]:                 {{site.gitrepo}}tree/main/rvt-patch-llvm

[Using KLEE]:                     {{site.baseurl}}{% post_url 2020-09-01-using-klee %}
[Using verification-annotations]: {{site.baseurl}}{% post_url 2020-09-02-using-annotations %}
[Using PropVerify]:               {{site.baseurl}}{% post_url 2020-09-03-using-propverify %}
[Install Crux]:                   {{site.baseurl}}{% post_url 2020-09-07-install-crux %}
[Using ARGV]:                     {{site.baseurl}}{% post_url 2020-09-09-using-argv %}
[Using FFI]:                      {{site.baseurl}}{% post_url 2020-12-11-using-ffi %}
[Profiling Rust]:                 {{site.baseurl}}{% post_url 2021-03-12-profiling-rust %}
[Hand vectorized Rust]:           {{site.baseurl}}{% post_url 2021-03-20-verifying-vectorized-code %}


[Measuring coverage]:             http://ccadar.blogspot.com/2020/07/measuring-coverage-achieved-by-symbolic.html
[KLEE testing CoreUtils]:         https://klee.github.io/tutorials/testing-coreutils/
[reid:hatra:2020]:                https://alastairreid.github.io/papers/HATRA_20/
[galea:arxiv:2018]:               https://alastairreid.github.io/RelatedWork/papers/galea:arxiv:2018/
[bornholt:oopsla:2018]:           https://alastairreid.github.io/RelatedWork/papers/bornholt:o opsla:2018/
[Verification Profiling]:         https://alastairreid.github.io/RelatedWork/notes/verification-profiling/
[leino:informatics:2001]:         https://alastairreid.github.io/RelatedWork/papers/leino:informatics:2001/

[Rust design for testability]:    https://alastairreid.github.io/rust-testability/
[Rust testing or verification]:   https://alastairreid.github.io/why-not-both/
[Verification competitions]:      https://alastairreid.github.io/verification-competitions/
