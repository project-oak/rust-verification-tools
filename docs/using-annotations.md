---
layout: page
title: "Using the `verification-annotations` crate"
permalink: /using-annotations/
---

_Note:
The recommended way to use KLEE is with the `propverify` library
as described [here](using-propverify.md).
This section describes how to use the `verification-annotations`
library instead of the `propverify` library:
in case you wonder how `propverify` works._

## The low-level verification API for C

Many tools for verifying C and Java programs share a common
API that looks a bit like this (where 'X' is any primitive type
like `int` or `char`).
(See [the Software Verification Competition website](https://sv-comp.sosy-lab.org/2020/rules.php)
for more details.)

```
/// create a non-deterministic value
X __VERIFIER_nondet_X();

/// add an assumption
void __VERIFIER_assume(int expression);

/// report an error
void __VERIFIER_error();
```

Typical usage is to:

1. Use `VERIFIER_nondet_X()` to create some non-deterministic values.
   The return value can potentially have any legal value of type `X`.

2. Use `VERIFIER_assume(<condition>)` to add constraints.
   For example, you might create a non-deterministic value `x` and then
   use `VERIFIER_assume(x < 1000)` to restrict the range of `x`.

3. Use `VERIFIER_error()` to report an error.

The job of the verifier is to report any possible choices of non-deterministic
values that can cause the program to reach a call to `VERIFIER_error()`.

(In practice, verifiers provide other functions and macros to print error
messages before quitting and you might use `VERIFIER_error` to define
a higher-level by, for example, defining an `assert` macro.)


## A low-level verification API for Rust

The [SMACK developers](http://smackers.github.io/) have been developing
some
[Rust benchmarks](https://github.com/soarlab/rust-benchmarks)
using a similar interfaces.
In adapting this interface for use with KLEE and PropVerify, we made
some small changes.

Our interface looks like this (but we should maybe change it to be
more like the original SMACK interface?):

```
pub trait AbstractValue : Sized {
    /// Create an abstract value of type `Self`
    fn abstract_value() -> Self;

    /// Create an abstract value satisfying a predicate `F`
    fn abstract_where<F: FnOnce(&Self) -> bool>(f: F) -> Self {
        let x = Self::abstract_value();
        assume(f(&x));
        x
    }
}

/// add an assumption
fn assume(c: bool);

/// report an error
fn abort(c: bool);
```

In addition, there are some utility functions that can be useful

```
/// Report an error and abort()
fn report_error(message: &str) -> !;

/// abort() if condition does not hold
fn assert(c: bool);

/// Indicate that the program is expected to panic with
/// an optional expected panic message.
/// This can be used to implement #[should_panic]
fn expect(message: Option<&str>);

/// Reject this path of execution
/// Equivalent to 'assume(false)'
fn reject() -> !;

/// Test whether the program is being 'replayed' using concrete
/// values discovered by an earlier verifier run.
fn is_replay() -> bool;
```

Of these, the most confusing are `reject()` and `is_replay()`.

The `reject()` function is useful when you want to create
a non-deterministic value `b` from another non-deterministic value `a` but some
values of `a` are not legal. For example, you might create a unicode character
like this:

```
let a : u32 = verifier::AbstractValue::abstract_value();
let b = match std::char::from_u32(a) {
    Some(r) => r,
    None => verifier::reject(),
}
```

What we are doing here is telling the verifier that it should ignore any
non-deterministic values that can lead to a call to `reject`.  This is similar
to rejecting an invalid random input value when fuzzing.

Some verifiers (such as KLEE) output some example input values that would
reproduce the error when they find a bug.
You can then rerun the program using those input values to try to
understand the bug. In particular, you might use a conventional debugger to
run the program.
The function `is_replay()` returns `true` when a program is being replayed on
concrete input values.


## A simple example

Verification tools for C are able to use this API to verify large, complex
C code such as [Linux kernel drivers](http://linuxtesting.org/ldv).
For this note though, we will limit ourselves to
the same simple example we
[used with `propverify`]({% link using-propverify.md %}).
This might be useful if you wonder how propverify is implemented
or if you prefer to use the more conventional verifier interface.

This code is in `demos/simple/klee` and the shell commands in this
file are in `demos/simple/klee/verify.sh`.

```
use verification_annotations as verifier;

fn t1() {
    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(1 <= a && a <= 1000);
    verifier::assume(1 <= b && b <= 1000);
    #[cfg(not(crux))]
    if verifier::is_replay() {
        eprintln!("Test values: a = {}, b = {}", a, b);
    }
    let r = a*b;
    verifier::assert!(1 <= r && r < 1000000);
}
```

This program is identical to the code that the `proptest!` macro expands to
for the [propverify example]({% link using-propverify.md %}).
It does the following

- generates two values `a` and `b` in the range [1..1000] and [1..1000]
    - creates two abstract/symbolic values `a` and `b`
    - adds constraints on the range of `a` and `b`
    - prints the values of `a` and `b` to stderr if in 'replay' mode
- multiplies `a` and `b`
- asserts that their product is in the range 1..1000000

To check this, we will follow the same steps that we
did in [the propverify example]({% link using-propverify.md %})
of creating a test crate and then using `cargo-verify`
to invoke KLEE.
(We cannot run this example with a fuzzer.)


The Rust compiler and KLEE are in the Dockerfile (see
[installation]({% link installation.md %})) so start the Docker image
by running

``` shell
cd demos/simple/annotations
../docker/run
```

All remaining commands in this file will be run in this docker
image.

(It is usually easiest to run this in one terminal while using
a separate editor to edit the files in another terminal.)

To try the above example, we will create a crate in which to experiment with this
code.


## Verifying with KLEE (and `cargo-verify`)

To verify the program using propverify, we use the `cargo-verify` script to
compile the program and verify the program using KLEE

```
cargo clean
cargo-verify --tests --verbose .
```

The program above has a deliberate error and KLEE reports the error

```
Running 1 test(s)
test t1 ... ERROR

test result: ERROR. 0 passed; 1 failed
VERIFICATION_RESULT: ERROR
```

To see what values of `a` and `b` cause the problem, we can replay
the program using concrete data values.

```
cargo-verify --tests --replay .
```

This produces the following additional output that shows that KLEE
found values `a = 1000` and `b = 1000` and it fails the test that `a*b < 1000000`.

```
    Test input try-verifier/kleeout-t1/test000001.ktest
    Test input try-verifier/kleeout-t1/test000002.ktest
          Finished dev [unoptimized + debuginfo] target(s) in 0.00s
           Running `target/x86_64-apple-darwin/debug/try-verifier`
      Test values: a = 1000, b = 1000
      thread 'main' panicked at 'assertion failed: 1 <= r && r < 1000000', src/main.rs:12:5
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Seeing these examples, it is obvious that the assertion should be changed to

```
    verifier::assert!(1 <= r && r <= 1000000);
```

With that fix, we can rerun KLEE and see that the test passes

```
cargo-verify --tests .
  Running 1 test(s)
  test t1 ... ok

  test result: ok. 1 passed; 0 failed
  VERIFICATION_RESULT: VERIFIED
```


## Verifying with Crux-mir

(if you fixed the assertion as discussed above, revert the fix)

Verify the program with Crux-mir

```
cargo crux-test --features verifier-crux
```

The program above has a deliberate error and Crux-mir reports the error

```
test try_verifier/1cqgh0ha::main[0]: FAILED

failures:

---- try_verifier/1cqgh0ha::main[0] counterexamples ----
Failure for MIR assertion at src/main.rs:14:5:
        VERIFIER: assertion failed: 1 <= r && r < 1000000
in try_verifier/1cqgh0ha::main[0] at ./lib/crucible/lib.rs:50:17
```

## Variations on a theme

We can get a slighly better understanding of this low-level API by modifying
the example a little.


### Emulating `#[should_panic]`

As a first variation, let's restore the original error and add a call to
`expect`:

```
use verification_annotations as verifier;

#[test]
fn main() {
    verifier::expect(Some("assertion failed"));
    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(1 <= a && a <= 1000);
    verifier::assume(1 <= b && b <= 1000);
    if verifier::is_replay() {
        eprintln!("Test values: a = {}, b = {}", a, b);
    }
    let r = a*b;
    verifier::assert!(1 <= r && r < 1000000);
}
```

The verifier now finds the failing assertion and confirms that this matches the
expected behavior.

```
../scripts/cargo-verify -v .
Checking try_verifier
Running 1 test(s)
     t1: Detected expected failure 'assertion failed: 1 <= r && r < 1000000' at src/main.rs:13:5
     t1: 2 paths
test t1 ... ok

test result: ok. 1 passed; 0 failed
VERIFICATION_RESULT: VERIFIED
```


### Triggering overflow

Another interesting variation is to relax the constraints on the input values.

For example, we might allow larger values for `a` and `b` or we might
just delete the assumptions.

```
use verification_annotations as verifier;

#[test]
fn t1() {
    // verifier::expect(Some("overflow"));
    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(1 <= a && a <= 1000000);
    verifier::assume(1 <= b && b <= 1000000);
    if verifier::is_replay() {
        eprintln!("Test values: a = {}, b = {}", a, b);
    }
    let r = a*b;
    verifier::assert!(1 <= r);
}
```

Verifying this program detects the overflow behaviour.

```
../scripts/cargo-verify -v --replay .
Running 1 test(s)
     t1: 2 paths
    Test input try-verifier/kleeout-t1/test000001.ktest
    Test input try-verifier/kleeout-t1/test000002.ktest
             Fresh verification-annotations v0.1.0 (verification-annotations)
             Fresh try-verifier v0.1.0 (try-verifier)
          Finished dev [unoptimized + debuginfo] target(s) in 0.00s
           Running `target/x86_64-apple-darwin/debug/try-verifier`
      Test values: a = 16384, b = 524288
      thread 'main' panicked at 'attempt to multiply with overflow', src/main.rs:12:13
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test t1 ... OVERFLOW

test result: OVERFLOW. 0 passed; 1 failed
VERIFICATION_RESULT: OVERFLOW
```

And, of course, we can uncomment the call to `verifier::expect()` to indicate
that this is an expected failure.
