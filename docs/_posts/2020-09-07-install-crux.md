---
layout: post
title: Crux-Mir installation
---

The best way to install crux (aka mir-verifier) is to follow the instructions on
[crux's GitHub page][Crux-MIR].

For convenience, instructions for installing crux and its dependencies are
provided below.

*[Note: the instructions here are probably out of date. We don't test with crux
often enough to spot when things change.]*


We are going to install Haskell, Rust, mir-json, Yices and crux.

Where possible `apt` is used. 
Everything else is installed under `$HOME`.


### Installing Haskell

``` shell
sudo apt install cabal-install ghc
cabal new-update
cabal user-config update
```

Make sure `PATH` includes `$HOME/.cabal/bin`.


### Installing Rust

Install rust using rustup.
mir-json requires nightly-2020-03-22 so we will get that.

``` shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly-2020-03-22 --force
rustup default nightly-2020-03-22
rustup component add --toolchain nightly-2020-03-22 rustc-dev
```

Make sure `PATH` includes `$HOME/.cargo/bin`.

### Install mir-json

``` shell
git clone git@github.com:GaloisInc/mir-json.git
cd mir-json
RUSTC_WRAPPER=./rustc-rpath.sh cargo install --locked
```

### Install Yices

``` shell
git clone git@github.com:SRI-CSL/yices2.git
cd yices2
autoconf
./configure --prefix="$HOME/.local"
make
make install
```

Make sure `PATH` includes `$HOME/.local/bin`

### Building Crux

``` shell
git clone git@github.com:GaloisInc/mir-verifier.git
cd mir-verifier
git submodule update --init
cabal v2-build
./translate_libs.sh
cabal v2-install crux-mir
```

### Shell init script

Add the following lines to your shell init script (assuming crux was cloned into
`$HOME/mir-verifier`).

``` shell
export PATH=$HOME/.local/bin:$HOME/.cabal/bin:$HOME/.cargo/bin:$PATH
export CRUX_RUST_LIBRARY_PATH=$HOME/mir-verifier/rlibs
```
### Testing

Run crux's test suite:

``` shell
cd mir-verifier
cabal v2-test
# [...]
# All 254 tests passed (322.16s)
# Test suite test: PASS
# Test suite logged to:
# [...]
# 1 of 1 test suites (1 of 1 test cases) passed.
```

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

