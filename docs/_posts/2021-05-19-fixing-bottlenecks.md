---
layout: post
title: Fixing bottlenecks in Rust verification
---

![KLEE logo](https://klee.github.io/images/klee.svg){: style="float: left; width: 10%; padding: 1%"}
It is inevitable that automatic verification tools will have performance
problems because they push up against ["the decidability
ceiling"][leino:informatics:2001]: trying to solve undecidable problems and
often getting away with it.
In [an earlier article][Profiling Rust], we looked at how to profile the
verification process to find which part of your program is causing the problem.
But that is only half the problem: we need to actually fix the problem.[^not-even-half]
So this article looks at one way that we can fix performance bottlenecks
when verifying Rust code using the [KLEE] symbolic execution tool.
In particular, it looks at using path-merging to overcome the path explosion problem.

[^not-even-half]:
    Arguably, profiling your code doesn't even solve half the problem.
    Until you have actually fixed it, you can't even be sure that the
    profiler has correctly identified the root case.

The way that symbolic execution tools like [KLEE] work is that
they enumerate all viable paths through your code and individually explore each one.
By "viable", what I mean is that each time KLEE finds a conditional branch, it
uses a solver to decide whether the whether the branch condition must be true,
must be false or could be either (depending on symbolic input data).
This third situation where a branch depends on the symbolic input data
is what distinguishes symbolic execution from conventional (concrete)
execution because the symbolic executor will continue executing both paths.

This property of symbolically executing multiple paths through a program
is a major cause of verification performance problems because
the number of viable paths through a program can grow exponentially.

For example, consider a loop containing a branch.

``` rust
for i in 0u32 .. N {
    if <condition> {
        x = x + 1;
    }
}
```

If the branch condition depends on symbolic values, then

- the first time round the loop, there will be two viable paths to execute
  so there will be two paths at the end of the loop
- the second time round the loop, each path will generate two viable paths to explore
  so there will be four paths at the end of the loop
- ...
- at the end of the Nth time round the loop, there will be 2^N paths
  to explore.

(For more detail, see [[bornholt:oopsla:2018]] and [[galea:arxiv:2018]].)

Model checkers avoid this problem by always merging paths whenever
they reach a control-flow join.
In cases like this loop, that is exactly the right thing to do; in others,
it causes an explosion in the state space.
But I'm using a symbolic execution tool so what can I do?

While looking through KLEE's documentation, I noticed that KLEE had some
support for manually merging paths by
calling the function `klee_open_merge` to start tracking path splits
and later calling `klee_close_merge` to merge paths back together.
There is not a lot of documentation but it seemed to be worth a shot
so extended our KLEE support to provide these functions and a convenience
macro to wrap a block of code.

``` rust
pub fn open_merge();
pub fn close_merge();

/// Coherent blocks don't fork execution during verification.
#[macro_export]
macro_rules! coherent {
    ( $body:block ) => {
        $crate::open_merge();
        $body;
        $crate::close_merge();
    };
}
```

To test these, I wrote a small test to create an array, set each
element of the array to a symbolic value

``` rust
use verification_annotations::prelude::*;

#[test]
fn test_merged() {
    // An array
    let mut a = [0u32; N];

    // Set each element of array to a symbolic value
    for i in &a {
        *i = u32::abstract_value();
    }

    // A loop containing two branches - this will cause a performance problem
    // for conventional symbolic execution.
    for x in a.iter() {
        verifier::coherent!{
            verifier::assume((5..10).contains(x) || (15..20).contains(x))
        }
    }

    // A true assertion about an arbitrary element of the array
    verifier::assert!(a[3] < 20);
}
```



Running this test on KLEE with

``` sh
    cargo verify --backend=klee --tests -vv --backend-flags=--use-merge |& grep 'generated tests'
```

should result in

```
    test_original: {"generated tests": 1024, "total instructions": 709566, "completed paths": 4093}
    test_merged: {"generated tests": 1, "completed paths": 41, "total instructions": 11852}
```

Indicating that the original version suffers from a path explosion and generates
more tests, executes more instructions and explores more paths than the merged version.



-----------

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
[regex bottleneck]:               {{site.gitrepo}}blob/main/demos/bottlenecks/regex/src/main.rs
[rust2calltree]:                  {{site.gitrepo}}tree/main/rust2calltree

[Using KLEE]:                     {{site.baseurl}}{% post_url 2020-09-01-using-klee %}
[Using verification-annotations]: {{site.baseurl}}{% post_url 2020-09-02-using-annotations %}
[Using PropVerify]:               {{site.baseurl}}{% post_url 2020-09-03-using-propverify %}
[Install Crux]:                   {{site.baseurl}}{% post_url 2020-09-07-install-crux %}
[Using ARGV]:                     {{site.baseurl}}{% post_url 2020-09-09-using-argv %}
[Using FFI]:                      {{site.baseurl}}{% post_url 2020-12-11-using-ffi %}
[Profiling Rust]:                 {{site.baseurl}}{% post_url 2021-03-12-profiling-rust %}

[Measuring coverage]:             http://ccadar.blogspot.com/2020/07/measuring-coverage-achieved-by-symbolic.html
[KLEE testing CoreUtils]:         https://klee.github.io/tutorials/testing-coreutils/
[galea:arxiv:2018]:               https://alastairreid.github.io/RelatedWork/papers/galea:arxiv:2018/
[bornholt:oopsla:2018]:           https://alastairreid.github.io/RelatedWork/papers/bornholt:oopsla:2018/
[Verification Profiling]:         https://alastairreid.github.io/RelatedWork/notes/verification-profiling/
[leino:informatics:2001]:         https://alastairreid.github.io/RelatedWork/papers/leino:informatics:2001/

[Rust design for testability]:    https://alastairreid.github.io/rust-testability/
[Rust testing or verification]:   https://alastairreid.github.io/why-not-both/
[Verification competitions]:      https://alastairreid.github.io/verification-competitions/
