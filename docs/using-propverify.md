# Using the `propverify` library to verify Rust programs

The goal of the tools and library in this repository is to let you verify
interesting things about non-trivial programs.
Unfortunately, interesting/non-trivial programs are too large for introducing
you to a tool so, for now, we will consider this trivial program.

```
proptest! {
    #[test]
    fn multiply(a in 1..=1000u32, b in 1..=1000u32) {
        let r = a*b;
        assert!(1 <= r && r < 1000000);
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
1. verify the crate using propverify and KLEE, and the cargo-verify script
1. verify the crate using propverify and Crux-mir

### Workaround for `compilation error`

Before we really get started...
The `cargo-verify` script sometimes gets confused by previous compilations.
If you get an error message involving `compilation error`, try running `cargo
clean` and try again.

Hopefully we'll make cargo-verify more robust soon.


## Creating a test crate

The Rust compiler and KLEE are in the Dockerfile (see
[installation](installation.md)) so start the Docker image
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

```
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
        assert!(1 <= r && r < 1000000);
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


## Verifying with `propverify` using KLEE

To verify the program using propverify, we use the `cargo-verify` script to
compile the program and verify the program using KLEE

```
cargo clean
cargo-verify . --tests --verbose
```

The program above has a deliberate error and KLEE reports the error

```
Running 1 test(s)
test multiply ... ERROR

test result: ERROR. 0 passed; 1 failed
VERIFICATION_RESULT: ERROR
```

While it is nice that it has found a bug, we need more detail
before we can understand and fix the bug.
After finding the bug, we can "replay" the test
to see some concrete data values that it fails on.

(Although it makes no difference in this small example, it is usually best to
use `--test=...` instead of `--tests` to focus on a single failing test at
a time.)


```
cargo-verify . --test=multiply --replay
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

```
    assert!(1 <= r && r <= 1000000);
```

With that fix, we can rerun KLEE and see that the test passes

```
cargo-verify . --tests --replay
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
