---
layout: post
title: "Profiling Rust verification"
---

Automatic formal verification is usually pushing up against what Leino calls ["the
decidability ceiling"][leino:informatics:2001]: pushing the tools beyond what
they can be guaranteed to solve in some reasonable time, taking the risk that
the tools will blow up, but often getting away with it.
But what can we do when the toast lands butter-side-down?
This is a summary of a quick investigation to find out if
anybody had any answers. Tl;dr: they do.

I found four relevant sources of information:

- Bornholt and Torlak, "[Finding code that explodes under symbolic
  evaluation][bornholt:oopsla:2018]", OOPSLA 2018
- Cadar and Kapus, "[Measuring the coverage achieved by symbolic
  execution][measuring coverage]", Blog 2020
- Galea et al., "[Evaluating manual intervention to address the challenges of
  bug finding with KLEE][galea:arxiv:2018]", arXiv 2018
- KLEE website, "[Tutorial on How to Use KLEE to Test GNU Coreutils (sections
  6 and 7)][KLEE testing CoreUtils]", 2018?

(Huge thanks to Sean Heelan for his [really useful response to my Twitter
request for suggestions](https://twitter.com/seanhn/status/1359244499793379335?s=20)
– resulting in the last two sources and which form the basis of most of the
rest of this post.)

These are great resources and, if you want to learn more about this topic,
I strongly recommend them.
They have one minor problem though, none of them are about verifying Rust code
which this website is all about.
To make things more Rusty, I decided to try to apply the [KLEE profiling
ideas][KLEE testing CoreUtils] to some code I had recently tried to verify and
that was scaling horribly.
The code called the [Rust regex crate][regex crate] and, like a lot of
string-processing code, it was behaving so badly that I could only work with
trivial length strings and carefully constrained regex queries.

I created [a trivial benchmark test harness][regex bottleneck] that matches
a sequence of digits and asserts that converting those digits to an integer is
less than some limit.
This version should fail because one possible result is 11: I also tested with
a non-failing variant that asserts "i <= 11".

``` rust
#[test]
fn regex_should_fail() {
    // generate a symbolic string of length 2
    let a = verifier::verifier_nondet_ascii_string(2);

    // assume the string consists of 0's and 1's
    verifier::assume(Regex::new(r"[0-1]{2}").unwrap().is_match(&a));

    // convert to int
    let i: u32 = a.parse().unwrap();

    // check the maximum value
    verifier::assert!(i < 11);
}
```

I expected that this would perform badly and I was not disappointed.
Running our [cargo verify][Using verification-annotations] tool to compile
the code for verification and then use KLEE to verify it spent distressingly
long in the verifier for such a short string.[^compilation-also-slow]

[^compilation-also-slow]:
    You will notice that compilation time is even worse than verification
    time.
    Part of this is due to the time fetching and compiling the regex library
    and part of it this is due to our tool not making good use of
    cargo's incremental compilation.
    We plan to fix this – but it is not our priority at the moment.

```
$ cargo verify --test=regex_should_fail -v
Using Klee as backend
Checking regex
  Building regex for verification
  Patching LLVM file for initializers and feature tests
  Checking regex_should_fail
Running 1 test(s)
test regex_should_fail ... ASSERT_FAILED
test result: ASSERT_FAILED. 0 passed; 1 failed
Build 47.085s
Verify 37.907s
Total 84.992s
VERIFICATION_RESULT: ASSERT_FAILED
```

The problem was probably in the Regex library, the question was whether I could
pin down which lines of code were responsible?

The [KLEE tutorial][KLEE testing CoreUtils] explains that KLEE generates
a profile of the verification process that gives you various metrics
about every line of code in your program (including any libraries that you
linked against).
And both [Galea et al.][galea:arxiv:2018]
and
[Bornholt and Torlak][bornholt:oopsla:2018]
highlight the performance impact of path splitting on the cost of symbolic execution.
So what I needed to do was run the [kcachegrind] tool
on the profile file generated by KLEE.

Before I could do that though, there was one small problem to take care
of. Looking inside the profile file, I could see that it was referring
to functions by their "mangled" names like 
`_ZN5regex17regex_should_fail17h11a2866ac324f18dE`
intead of human-readable names like
`regex::regex_should_fail`.
To fix this, I wrote [a small demangling tool][rust2calltree] that could demangle the
profile file.
With this in place, I could demangle the profile and view it like this

```
$ rust2calltree kleeout/regex_should_fail/run.istats -o callgrind.out
$ kcachegrind
```

This produced a window that looked like this
(I have selected the "forks" metric and clicked on the most expensive function
that is not my test harness.)

![Fork profile of regex_should_fail]({{site.baseurl}}/images/profiling-regex.png)

This profile shows that the function that is most responsible for
the path explosion in my benchmark is the function
`aho_corasick::ahocorasick::AhoCorasick::find`.

This was a surprise to me – I was expecting that it would be complaining about
part of the regex crate that my benchmark was calling.
I did a little digging and I quickly found out that the [aho-corasick crate]
was a fast string matching library used by the [regex crate]
for simple parts of regexps.
Since my regexp was quite simple, it seemed reasonable that this crate would
be doing most of the work and therefore would be potentially part of the
problem.
(I didn't confirm that this really was the problem.
To do that, I would probably have to modify the Aho-Corasick crate to
fix the problem – which I have not tried doing, yet.)


## Summary

Verification tools are very likely to hit performance bottlenecks because
they are designed to push against the [decidability ceiling][leino:informatics:2001].

There has not been a lot of work on profiling to help you identify what
part of your code is responsible for any performance problems you hit
when trying to verify your code but the papers/tutorials that I found were
very useful and are worth reading.

With the aid of a [little script][rust2calltree], I was able to use
the instructions in the [KLEE tutorial][KLEE testing CoreUtils]
to profile a small benchmark and it produced results that seemed entirely
plausible and that highlighted some code that I would probably not have
looked at first.

The task of confirming that the code identified by profiling can be
"fixed" is something that I leave for future work.

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

[Measuring coverage]:             http://ccadar.blogspot.com/2020/07/measuring-coverage-achieved-by-symbolic.html
[KLEE testing CoreUtils]:         https://klee.github.io/tutorials/testing-coreutils/
[galea:arxiv:2018]:               https://alastairreid.github.io/RelatedWork/papers/galea:arxiv:2018/
[bornholt:oopsla:2018]:           https://alastairreid.github.io/RelatedWork/papers/bornholt:oopsla:2018/
[Verification Profiling]:         https://alastairreid.github.io/RelatedWork/notes/verification-profiling/
[leino:informatics:2001]:         https://alastairreid.github.io/RelatedWork/papers/leino:informatics:2001/

[Rust design for testability]:    https://alastairreid.github.io/rust-testability/
[Rust testing or verification]:   https://alastairreid.github.io/why-not-both/
[Verification competitions]:      https://alastairreid.github.io/verification-competitions/