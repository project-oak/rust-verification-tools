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
2. fuzz the crate using proptest
3. verify the crate using propverify and the cargo-verify script

### Workaround for `compilation error`

Before we really get started...
The `cargo-verify` script sometimes gets confused by previous compilations.
If you get an error message involving `compilation error`, try running `cargo
clean` and try again.

Hopefully we'll make cargo-verify more robust soon.


## Creating a test crate

```
cargo new try-propverify
cd try-propverify

cat >> Cargo.toml  << "EOF"

[target.'cfg(verify)'.dependencies]
propverify = { path="../propverify" }

[target.'cfg(not(verify))'.dependencies]
proptest = { version = "*" }

[features]
verifier-klee = ["propverify/verifier-klee"]
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


## Verifying with `propverify`

To verify the program using propverify, we use the `cargo-verify` script to
compile the program and verify the program using KLEE

```
cargo clean
../scripts/cargo-verify . --tests --verbose
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
../scripts/cargo-verify . --test=multiply --replay
```

This produces additional output that shows that KLEE
explored two paths through the program: one that passes and one that fails.

The first path has value `a = 1` and `b = 1` and it passes the test.

```
    Test input try-it-out/kleeout-multiply/test000001.ktest

      running 1 test
        Value a = 1
        Value b = 1
      test multiply ... ok

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The second path has value `a = 1000` and `b = 1000` and it fails the test that `a*b < 1000000`.

```
    Test input try-it-out/kleeout-multiply/test000002.ktest
          Finished test [unoptimized + debuginfo] target(s) in 0.02s
           Running target/x86_64-apple-darwin/debug/deps/try_it_out-bb37abd6d1dc60ef
      thread 'multiply' panicked at 'assertion failed: 1 <= r && r < 1000000', src/main.rs:10:9
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
      error: test failed, to rerun pass '--bin try-it-out'

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
../scripts/cargo-verify . --tests --replay
Running 1 test(s)
test multiply ... ok

test result: ok. 1 passed; 0 failed
VERIFICATION_RESULT: VERIFIED
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
