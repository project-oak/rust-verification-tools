# Using Rust's foreign function interface

_[Note:
You should not need any of the information in this note to
use `propverify` if you are using the `cargo-verify` script.
These instructions are mostly useful if you want to create your own
tools or if you hit problems.]_

Rust is able to call C code using the FFI (Foreign Function Interface).
This note describes how to verify crates that consist of a mixture of
Rust code and C code that is built using a
[build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
such as
[Alex Crichton's cc-rs crate](https://github.com/alexcrichton/cc-rs).
(If your crate calls a separate C library (e.g., libX11 or libSSL),
you will need to do a bit more, although this note may be a useful starting
point.)

## A simple example

For the sake of an example, we will consider a simple Rust+C program
consisting of a C library `bar.c`

``` C
#include <stdint.h>

int32_t bar_function(int32_t x) {
    return x+1;
}
```

Rust test file `src/main.rs`

``` Rust
#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

#[link(name = "bar_library")]
extern {
    fn bar_function(x: i32) -> i32;
}

fn bar(x: i32) -> i32 {
    unsafe {
        bar_function(x)
    }
}

proptest!{
    fn main(i in any::<i32>()) {
        prop_assert!(bar(i) != i)
    }
}

proptest! {
    #[test]
    fn inequal(x in any::<i32>()) {
        prop_assert!(bar(x) != x)
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn greater(x in any::<i32>()) {
        prop_assert!(bar(x) > x)
    }
}
```

A [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
that uses
[Alex Crichton's cc-rs crate](https://github.com/alexcrichton/cc-rs)
to compile the C library.

``` Rust
fn main() {
    cc::Build::new()
        .file("bar.c")
        .compile("bar_library");
}
```

And, finally, a cargo file

``` toml
[dependencies]
libc = "0.2"

[target.'cfg(not(verify))'.dependencies]
proptest = { version = "0.10" }

[target.'cfg(verify)'.dependencies]
propverify = { path="/home/rust-verification-tools/propverify" }

[features]
verifier-klee = ["propverify/verifier-klee"]
verifier-crux = ["propverify/verifier-crux"]

[build-dependencies]
cc = "1.0"
```

This code is all in the [demos/simple/ffi directory](../demos/simple/ffi).


## Testing FFI code with proptest

We can use the
[proptest](https://github.com/AltSysrq/proptest) library to test the example
code.

``` shell
docker/run
cd demos/simple/ffi
cargo test --tests
```

which should produce output like the following

```
running 2 tests
test greater ... FAILED
test inequal ... ok

...

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

The failure we detected is not a failure in the test but, instead, a limitation of proptest.
The test `greater` will fail on exactly one input value (`0x7fff_ffff`)
but proptest checks properties by testing with random numbers so it is very
unlikely to find the one input that fails the test.


We can "fix" this failure by commenting out the following line

``` rust
    #[should_panic(expected = "assertion failed")]
```

(Alternative fixes might be to use a strategy that biases the random
numbers to very large and very small numbers or to restrict the range of `x`
to fix the property.)


_[Note: for consistency, the above instructions run proptest in the docker
environment – but this is probably not necessary since it is easy to install
Rust and use proptest on most platforms.]_


## Verifying FFI code with `cargo verify`

The simplest way to verify our example is using `cargo verify`.

``` shell
docker/run cargo test demos/simple/ffi -v -r --clean
```

which should produce output like this which shows that two tests behaved as expected
and, in particular, the property `greater` does not hold for input `x = 2147483647`.

```
../../../docker/run cargo verify . --tests -r --clean
[sudo] password for adreid:
Running 2 test(s)
test inequal ... ok
    Test input /usr/local/google/home/adreid/rust/rvt/demos/simple/ffi/kleeout-greater/test000002.ktest
         Compiling ffi v0.1.0 (/usr/local/google/home/adreid/rust/rvt/demos/simple/ffi)
          Finished test [unoptimized + debuginfo] target(s) in 0.92s
           Running target/x86_64-unknown-linux-gnu/debug/deps/ffi-8e3160ef933d2253
      VERIFIER_EXPECT: should_panic(expected = "assertion failed")
      VERIFIER: panicked at 'assertion failed: bar(x) > x', src/main.rs:34:9
      error: test failed, to rerun pass '--bin ffi'

      Caused by:
        process didn't exit successfully: `/usr/local/google/home/adreid/rust/rvt/demos/simple/ffi/target/x86_64-unknown-linux-gnu/debug/deps/ffi-8e3160ef933d2253 greater --nocapture` (signal: 6, SIGABRT: process abort signal)

      running 1 test
        Value x = 2147483647
test greater ... ok

test result: ok. 2 passed; 0 failed
VERIFICATION_RESULT: VERIFIED
```

Aside: the flags used are often useful flags to use with cargo-verify:

- `-r` replays the tests to show the failing input values
- `-v` increases verbosity a little (you can use multiple `-v` flags to increase
  verbosity)
- `--clean` runs `cargo clean` before running the test.
  We use this flag in instructions like this to make sure that
  you see the same results that we get – but you should be
  able to omit it.


## How we verify Rust crates that use FFI


If you just want to verify crates that use FFI and it is working reliably for
you, then you don't need to know how the above works.
But if you are trying to port our tools to some other verification backend
or if you run into problems, read this section to learn how cargo-verify
handles FFI code.


In [docs/using-klee.md](using-klee.md) we saw the following

- The Rust compiler is based on [LLVM](https://llvm.org/).
- Verifiers such as [KLEE](http://klee.github.io/) can verify programs built using LLVM.
- If we compile Rust with the flags `"-Clto -Cembed-bitcode=yes --emit=llvm-bc"`
  then the Rust compiler will generate an intermediate file consisting of all
  the Rust code from the current crate and all of the crates that it
  transitively depends on.

For example, if we run the following commands

``` shell
RUSTFLAGS="-Clto -Cembed-bitcode=yes --emit=llvm-bc --cfg=verify" cargo build --features=verifier-klee
klee --libc=klee --silent-klee-assume --warnings-only-to-file target/debug/deps/ffi-*.bc
```

We will see the crate being compiled and then KLEE 
generates some warnings (that we can ignore) and the
following error which shows that the LLVM file that we are verifying
does not include code for the C function `bar_function`.

```
KLEE: ERROR: src/main.rs:13: failed external call: bar_function
```

To fix this error, we need to change how the C code is compiled and
then we need to link the resulting file to the bitcode file containing the Rust
code.
The complete sequence of commands to do this is the following monster:

``` shell
CC=clang-10 \
  CRATE_CC_NO_DEFAULTS=true \
  CFLAGS="-flto=thin" \
  RUSTFLAGS="-Clto -Cembed-bitcode=yes --emit=llvm-bc --cfg=verify -Clinker-plugin-lto -Clinker=clang-10 -Clink-arg=-fuse-ld=lld" \
  cargo build --features=verifier-klee
llvm-link -o t.bc target/debug/deps/ffi-*.bc target/debug/build/ffi-*/out/*.o
klee --libc=klee --silent-klee-assume --warnings-only-to-file t.bc
```

Let's go through these commands slowly.

### Generating LLVM bitcode for the C code

The first parts of that command are as follows:

- Use the [Clang](https://clang.llvm.org/) C compiler to compile the C code

  ``` shell
  CC=clang-10
  ```

  (We use version 10 of `clang` because we build KLEE and Rust using version 10
  of LLVM.)

- Turn off some default flags that `cc-rs` normally uses
  (in particular, `-ffunction-sections` and `-fdata-sections`)

  ``` shell
  CRATE_CC_NO_DEFAULTS=true
  ```

- Generate a bitcode file from the C code

  ``` shell
  CFLAGS="-flto=thin"
  ```

  (It is (probably) also possible to use the flag `"-fembed-bitcode"` – but that
  makes the linking step more complex.)

We can confirm that these flags have generated a bitcode file by looking at the
file generated

``` shell
llvm-dis < target/debug/build/ffi-*/out/*.o
```

which will disassemble the bitcode file and show the LLVM code for `bar_function`.


### Using the LLVM linker

An unfortunate side-effect of using the above flags is that the normal linking step fails
because we are now generating LLVM bitcode for the C file instead of x86 code.
Although we mostly want LLVM bitcode, it is useful to have x86 code as well
because it is used by the 'replay' mechanism that we use to display failing input values.

We can fix the linking problem by adding the following flags to `RUSTFLAGS`.
This uses an LLVM linker instead of an ELF linker.

``` shell
-Clinker-plugin-lto -Clinker=clang-10 -Clink-arg=-fuse-ld=lld
```


### Linking the bitcode files

With the above flags, `cargo build`
compiles the C code to generate LLVM bitcode;
compiles the Rust code to generate LLVM bitcode;
links all the bitcode together;
compiles all the bitcode to x86;
and then generates a binary in `target/debug/ffi`.

Unfortunately, `cargo build` does not save the result of linking all the bitcode together
so we need to re-link the bitcode by invoking the LLVM linker explicitly.

```
llvm-link -o t.bc target/debug/deps/ffi-*.bc target/debug/build/ffi-*/out/*.o
```

This generates a file `t.bc` that contains all the LLVM bitcode for the C and
Rust code.

(This command will also produce a warning message about linking modules with
different target triples. This warning seems to be benign.)


### Running KLEE

Having generated a bitcode file, we can run KLEE as before

```
klee --libc=klee --silent-klee-assume --warnings-only-to-file t.bc
```

This will generate further benign(?) warnings about linking module with different target
triples and then the following output

```
KLEE: done: total instructions = 11768
KLEE: done: completed paths = 1
KLEE: done: generated tests = 1
```

indicating that KLEE verified 'main' and found one path through the code and
that path did not produce an error.

(If you want to check that KLEE could detect a problem if there was one,
try changing the assertion in `main` to `prop_assert!(bar(i) > i)`.)


### Running the `#[test]`s

If we want to run the tests in `src/main.rs` instead, we need to compile the
code slightly differently by adding the flag `--tests` to the `cargo build` command.

``` shell
cargo clean
CC=clang-10 CRATE_CC_NO_DEFAULTS=true CFLAGS="-flto=thin" \
  RUSTFLAGS="-Clto -Cembed-bitcode=yes --emit=llvm-bc --cfg=verify -Clinker-plugin-lto -Clinker=clang-10 -Clink-arg=-fuse-ld=lld" \
  cargo build --features=verifier-klee --tests
llvm-link -o t.bc target/debug/deps/ffi-*.bc target/debug/build/ffi-*/out/*.o
```

_[Note: it is essential to use `cargo build` in the above to delete the previous build.]_

We now have to find the names of the test functions `ffi::greater` and `ffi::inequal`.
Under the Rust compiler's name mangling scheme, Rust symbols always start with `_ZN`
and each component of the name is preceded by its length and ends in some random hash value.
So we are looking for symbols that start with `_ZN3ffi7greater` and `_ZN3ffi7inequal`.

``` shell
$ llvm-nm t.bc | grep '_ZN3ffi'
---------------- t _ZN3ffi3bar17h1325ecd6242160f7E
---------------- t _ZN3ffi4main17hca1ff028b832be3fE
---------------- t _ZN3ffi7greater17h8b653326034fc774E
---------------- t _ZN3ffi7greater28_$u7b$$u7b$closure$u7d$$u7d$17hdbf5c1e49e767973E
---------------- t _ZN3ffi7inequal17hdad7e86ce2cd07dbE
---------------- t _ZN3ffi7inequal28_$u7b$$u7b$closure$u7d$$u7d$17h490d0eddcf06f93bE
```

and now we can run KLEE

```
klee --libc=klee --silent-klee-assume --warnings-only-to-file --entry-point=_ZN3ffi7inequal17hdad7e86ce2cd07dbE t.bc
klee --libc=klee --silent-klee-assume --warnings-only-to-file --entry-point=_ZN3ffi7greater17h8b653326034fc774E t.bc
```

The first of these produces similar output to before indicating that the `inequal` property holds.

```
KLEE: done: total instructions = 246
KLEE: done: completed paths = 1
KLEE: done: generated tests = 1
```

The second KLEE run is more interesting because it detects an error in the `greater` property.

```
VERIFIER_EXPECT: should_panic(expected = "assertion failed")
VERIFIER: panicked at 'assertion failed: bar(x) > x', src/main.rs:34:9
KLEE: ERROR:
/home/rust-verification-tools/verification-annotations/src/klee.rs:95: abort
failure
KLEE: NOTE: now ignoring this error at this location

KLEE: done: total instructions = 18312
KLEE: done: completed paths = 2
KLEE: done: generated tests = 2
```

