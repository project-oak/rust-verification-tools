---
layout: post
title: "Using the `propverify` library to verify Rust programs"
permalink: /using-propverify/
---

The goal of the tools and library in this repository is to let you verify
interesting things about non-trivial programs.
Unfortunately, interesting/non-trivial programs are too large for introducing
you to a tool so, for now, we will consider this trivial program.

```rust
proptest! {
    #[test]
    fn multiply(a in 1..=1000u32, b in 1..=1000u32) {
        let r = a*b;
        prop_assert!(1 <= r && r < 1000000);
    }
}
```

This program

- generates two values `a` and `b` in the range [1..1000]
- multiplies `a` and `b`
- asserts that their product is in the range 1..1000000

In this note, we shall do the following to check this code

1. create a test crate
1. fuzz the crate using proptest
1. verify the crate using propverify and cargo-verify
1. verify the crate using propverify and Crux-mir

### Workaround for `compilation error`

Before we really get started...
The `cargo-verify` tool sometimes gets confused by previous compilations.
If you get an error message involving `compilation error`, try running `cargo
clean` and try again.

Hopefully we'll make cargo-verify more robust soon.


## Creating a test crate

The Rust compiler, KLEE, and Seahorn are in the Dockerfile (see
[installation][RVT installation]) so start the Docker image
by running

``` shell
../docker/run
```

All remaining commands in this file will be run in this docker
image.

(It is usually easiest to run this in one terminal while using
a separate editor to edit the files in another terminal.)

To try the above example, we will create a crate in which to experiment with this
code.

```shell
cargo new try-propverify
cd try-propverify

cat >> Cargo.toml  << "EOF"

[target.'cfg(verify)'.dependencies]
propverify = { path="/home/rust-verification-tools/propverify" }

[target.'cfg(not(verify))'.dependencies]
proptest = { version = "*" }

[features]
verifier-klee = ["propverify/verifier-klee"]
verifier-crux = ["propverify/verifier-crux"]
verifier-seahorn = ["propverify/verifier-seahorn"]
EOF

cat > src/main.rs  << "EOF"
#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

proptest! {
    #[test]
    fn multiply(a in 1..=1000u32, b in 1..=1000u32) {
        let r = a*b;
        prop_assert!(1 <= r && r < 1000000);
    }
}
EOF
```


## Fuzzing with `proptest`

Since proptest is a normal library, we can fuzz the
program using `cargo test`.

This will generate output a bit like this

```
running 1 test
test multiply ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Which means that the fuzzer has failed to find a bug.

(As we shall see in a moment, the test actually has a bug.
We have deliberately chosen ranges that are large enough that
the current version of proptest is unlikely to find the bug.)


## Verifying with `propverify` using `cargo-verify`

To verify the program using propverify, we use the `cargo-verify` tool to
compile the program and verify the program using one of the verification
backends (currently we support KLEE and Seahorn).

```shell
cargo clean
cargo-verify --backend=klee --tests --verbose
```

The program above has a deliberate error and KLEE reports the error

```
Running 1 test(s)
test multiply ... ERROR

test result: ERROR. 0 passed; 1 failed
VERIFICATION_RESULT: ERROR
```

Running `cargo-verify` with the option `--backend=seahorn` will give the same
result, but with Seahorn as the verification backend, instead of KLEE.

While it is nice that it has found a bug, we need more detail
before we can understand and fix the bug.
After finding the bug, we can "replay" the test
to see some concrete data values that it fails on.

(The Seahorn backend does not support replay at the moment)

(Although it makes no difference in this small example, it is usually best to
use `--test=...` instead of `--tests` to focus on a single failing test at
a time.)

```shell
cargo-verify --backend=klee --test=multiply --replay
```

This produces additional output that shows that KLEE
explored two paths through the program: one that passes and one that fails.

The first path has value `a = 1` and `b = 1` and it passes the test.

```
    Test input try-propverify/kleeout-multiply/test000001.ktest

      running 1 test
        Value a = 1
        Value b = 1
      test multiply ... ok

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The second path has value `a = 1000` and `b = 1000` and it fails the test that `a*b < 1000000`.

```
    Test input try-propverify/kleeout-multiply/test000002.ktest
          Finished test [unoptimized + debuginfo] target(s) in 0.02s
           Running target/x86_64-apple-darwin/debug/deps/try_propverify-bb37abd6d1dc60ef
      thread 'multiply' panicked at 'assertion failed: 1 <= r && r < 1000000', src/main.rs:10:9
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
      error: test failed, to rerun pass '--bin try-propverify'

      running 1 test
        Value a = 1000
        Value b = 1000
      test multiply ... FAILED

      failures:

      failures:
          multiply

      test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

Seeing this failing example, it is obvious that the comparision '<' should be changed to '<='

```rust
    prop_assert!(1 <= r && r <= 1000000);
```

With that fix, we can rerun KLEE and see that the test passes

```shell
cargo-verify --backend=klee --tests --replay
Running 1 test(s)
test multiply ... ok

test result: ok. 1 passed; 0 failed
VERIFICATION_RESULT: VERIFIED
```

## Verifying with `propverify` using Crux-mir

[The following does not run in docker at present]

(if you fixed the assertion as discussed above, revert the fix)

To verify the program using propverify and Crux-mir

```
cargo clean
RUSTFLAGS='--cfg verify' cargo crux-test --features verifier-crux
```

The program above has a deliberate error and Crux-mir reports the error

```
test try_propverify/17clqo26::multiply[0]: FAILED

failures:

---- try_propverify/17clqo26::multiply[0] counterexamples ----
Failure for panicking::begin_panic, called from try_propverify/17clqo26::multiply[0]
in try_propverify/17clqo26::multiply[0] at internal
```

As the test uses Rust's `std::assert` macro which invokes `panic!` the failure
Crux-mir detects is the call to panic.
To get a more descriptive failure we can replace `assert!` in src/main.rs with
`verifier::assert!`, and run the verifier again

```
RUSTFLAGS='--cfg verify' cargo crux-test --features verifier-crux

```

This time we get the following report which shows the assertion that failed and
its line number


```
test try_propverify/17clqo26::multiply[0]: FAILED

failures:

---- try_propverify/17clqo26::multiply[0] counterexamples ----
Failure for MIR assertion at src/main.rs:10:9:
        VERIFIER: assertion failed: 1 <= r && r < 1000000
in try_propverify/17clqo26::multiply[0] at ./lib/crucible/lib.rs:50:17
```

## Which is better: fuzzing or verification?

This example is intended as a quick introduction to using
property-based testing using either fuzzing (with proptest)
or verification tools (with propverify).

In this carefully chosen example, proptest missed
a bug that propverify found.
We could also have chosen an example where
proptest quickly finds a bug but propverify runs for
hours or days without producing a result.

The point of this example (and the point of making propverify
compatible with proptest) is not that one approach is better
than another but that they are complementary and
you should use both fuzzing and verification tools.


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

