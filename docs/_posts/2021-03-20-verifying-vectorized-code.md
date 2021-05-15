---
layout: post
title: Verifying hand-vectorized Rust code
---

![Rust logo](https://www.rust-lang.org/static/images/rust-logo-blk.svg){: style="float: left; width: 10%; padding: 1%"}
One of the major themes in our work on Rust verification is
eliminating verification obstacles: things that mean that you
can't even run a verification tool on your code.
So we have worked on how to [verify cargo crates][using KLEE],
how to [verify code that uses the Rust FFI][using FFI]
and how to [verify programs with command line arguments][using ARGV].
One recurring obstacle is that some code depends on processor-specific
intrinsic functions.
For example the [Aho-Corasick crate] supports fast string searches
using SIMD acceleration and uses intrinsics to access the x86 architecture's
AVX2 or SSE2 vector extensions if they are available.
Although verification tools *could* support these intrinsics,
most of them do not -- so if your program uses Aho-Corasick (or
any crate that depends on it like the [regex crate]), then you won't be
able to verify your program.

**[Since writing this post, we have realized that the approach described here
does not work very well. We now favour the much more effective "emulate, don't
eliminate" approach [described here][Verifying vectorized Rust revisited]:
using a SIMD emulation library to replace SIMD intrinsics.]**

The solution to this problem lies in the cause of the problem.
Verification tools don't implement these intrinsics because they
are not available on all processors.
For the same reason, any reasonably portable library must have a way
of disabling use of these intrinsics.  Typically, it has a portable "slow path"
that can be used if the vectorized "fast path" is not supported.
So the solution is to force the library to always use the more
portable slow path.[^is-it-slow] [^but-youre-not-verifying-the-real-code]

[^is-it-slow]:
    There's an interesting question (that I won't explore)
    about whether the simpler control structures of vectorized
    code might actually make them simpler to verify?
    In other words, maybe we should try supporting
    vector intrinsics in case it makes verification easier/faster?

[^but-youre-not-verifying-the-real-code]:
    You might object that now I'm not verifying the code that actually runs/ships.
    This would be a serious problem if my goal was to verify the
    vectorized library: I would ideally want to check that the fast path
    and the slow path are equivalent.

    On the other hand, if I am verifying a client library that calls the vectorized library,
    and I am more interested in bugs in the client library, then
    I might be happy to use the slow path as a proxy specification
    for what the fast path does while, reluctantly, accepting the fact
    that a bug in the vectorized code could break my verified program.

The way that Rust programs dynamically decide whether a given instruction
extension is available or not is to use the [std-detect] crate
and to use macros like this which detect whether the AVX2 extension
is available.

``` rust
let has_avx = is_x86_feature_detected!("avx2");
```

This macro expands to code that calls [std::std_detect::detect::check_for]:
a function that returns 'true' only if a processor extension is present.

``` rust
fn check_for(x: Feature) -> bool
```

What if we could arrange for this function to always return 'false'
no matter what feature you ask for?

As part of our verification toolchain, we had already had to write
[a tool][rvt-patch-llvm] that would preprocess LLVM bitcode files.
We use this to [collect initializers and make sure that they are run
before `main`][using ARGV] using this command

``` shell
$ rvt-patch-llvm --initializers foo.bc -o bar.bc
```

Most of the effort of writing a tool like that is in figuring out how to read,
write and traverse LLVM IR.  The actual transformation itself is fairly easy.
So, it's pretty easy to extend the tool to look for a function with particular
name and replace its body with `return false`.

So, now, the command

``` shell
$ rvt-patch-llvm --initializers --features foo.bc -o bar.bc
```

turns an LLVM bitcode file 'foo.bc' that uses processor-specific intrinsics
on its fast path into a bitcode file 'bar.bc' that does not.

If you are using a symbolic execution tool like [KLEE] that tests the feasibility of a
path before going down it, that will be enough to fix
the problem because execution will never hit the fast path with its unsupported intrinsics.
If you are using a model checker like [SeaHorn], that examines paths before testing their
feasibility, I suspect that you will need to work a bit harder and
use the LLVM optimization tool to perform constant propagation and dead-code
elimination to completely remove all mention of these intrinsics from the bitcode file.

## Summary

Most verification tools don't implement every single processor specific intrinsic supported
by compilers: especially not intrinsics that are only used for performance optimizations.

Anybody trying to write a portable library has added fallback code that can be used on processors
that don't support those intrinsics.

So all you have to do is ensure that the verifier only has to consider the
portable version of the code and you can verify code that uses hand-vectorized libraries.

------------------

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
[compatibility-test]:             {{site.gitrepo}}blob/main/compatibility-test/src
[demos/simple/ffi directory]:     {{site.gitrepo}}blob/main/demos/simple/ffi/
[CONTRIBUTING]:                   {{site.gitrepo}}blob/main/CONTRIBUTING.md
[LICENSE-APACHE]:                 {{site.gitrepo}}blob/main/LICENSE-APACHE
[LICENSE-MIT]:                    {{site.gitrepo}}blob/main/LICENSE-MIT
[regex bottleneck]:               {{site.gitrepo}}blob/main/demos/bottlenecks/regex/src/main.rs
[rust2calltree]:                  {{site.gitrepo}}tree/main/rust2calltree
[rvt-patch-llvm]:                 {{site.gitrepo}}tree/main/rvt-patch-llvm

[Using KLEE]:                     {{site.baseurl}}{% post_url 2020-09-01-using-klee %}
[Using verification-annotations]: {{site.baseurl}}{% post_url 2020-09-02-using-annotations %}
[Using PropVerify]:               {{site.baseurl}}{% post_url 2020-09-03-using-propverify %}
[Install Crux]:                   {{site.baseurl}}{% post_url 2020-09-07-install-crux %}
[Using ARGV]:                     {{site.baseurl}}{% post_url 2020-09-09-using-argv %}
[Using FFI]:                      {{site.baseurl}}{% post_url 2020-12-11-using-ffi %}
[Profiling Rust]:                 {{site.baseurl}}{% post_url 2021-03-12-profiling-rust %}
[Verifying vectorized Rust revisited]: {{site.baseurl}}{% post_url 2021-03-20-verifying-vectorized-code %}

[Measuring coverage]:             http://ccadar.blogspot.com/2020/07/measuring-coverage-achieved-by-symbolic.html
[KLEE testing CoreUtils]:         https://klee.github.io/tutorials/testing-coreutils/
[galea:arxiv:2018]:               https://alastairreid.github.io/RelatedWork/papers/galea:arxiv:2018/
[bornholt:oopsla:2018]:           https://alastairreid.github.io/RelatedWork/papers/bornholt:o opsla:2018/
[Verification Profiling]:         https://alastairreid.github.io/RelatedWork/notes/verification-profiling/
[leino:informatics:2001]:         https://alastairreid.github.io/RelatedWork/papers/leino:informatics:2001/

[Rust design for testability]:    https://alastairreid.github.io/rust-testability/
[Rust testing or verification]:   https://alastairreid.github.io/why-not-both/
[Verification competitions]:      https://alastairreid.github.io/verification-competitions/
