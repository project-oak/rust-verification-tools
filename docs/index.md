---
layout: home
regenerate: true
---

[RVT][RVT git repo] is a collection of research tools/libraries to support both static verification
(formal verification) and dynamic verification (testing) of Rust.
You can download RVT from
[https://github.com/project-oak/rust-verification-tools/](https://github.com/project-oak/rust-verification-tools/);
RVT is dual-licensed ([Apache][LICENSE-APACHE]/[MIT][LICENSE-MIT]) so that you can use and adapt our 
code for your own tools.

<a class="github-fork-ribbon" href="{{site.gitrepo}}" data-ribbon="Fork me on GitHub" title="Fork me on GitHub">Fork me on GitHub</a>


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

