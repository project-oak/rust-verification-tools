---
layout: post
title: "Using KLEE"
permalink: /using-klee/
---

![KLEE logo](https://klee.github.io/images/klee.svg){: style="float: left; width: 10%; padding: 1%"}
One of the most straightforward and most solid automatic verification tools is the [KLEE]
symbolic execution tool that can be used to search for bugs in programs.
KLEE was originally developed for C but, because [KLEE] and the Rust compiler
are both based on the [LLVM] platform, it is possible to use KLEE
with Rust programs.

The steps involved in using
KLEE to verify Rust programs are:

1. Compile the program to generate LLVM bitcode.
2. Run KLEE on the bitcode file.
3. Examine the output of KLEE.

With minor variations, it should be possible to adapt the basic steps of
compiling Rust programs to generate LLVM for use with other LLVM-based
verification tools.

_Note:
The recommended way to use KLEE is with the `propverify` library
and the `cargo-verify` script
as described [here][Using PropVerify].
This note describes how to use the KLEE directly
in case you wonder how `cargo-verify` works or want
to add support for a different tool.
This is going to be a fairly low-level description and most
people will be happier not knowing how the sausage is made.
(Since there are so many low-level details, there is a risk
that we will forget to update this document so you may have to [use the
source][cargo-verify source].)_


## A small test program

As a running example, we will use the same example
that we used to explain [how to use the verification-annotations
crate][Using verification-annotations].

This code is in [demos/simple/klee/src/main.rs] and the shell commands in this
file are in [demos/simple/klee/verify.sh].

``` rust
use verification_annotations::prelude::*;

fn main() {
    let a = u32::abstract_value();
    let b = u32::abstract_value();
    verifier::assume(1 <= a && a <= 1000);
    verifier::assume(1 <= b && b <= 1000);
    if verifier::is_replay() {
        eprintln!("Test values: a = {}, b = {}", a, b);
    }
    let r = a*b;
    verifier::assert!(1 <= r && r < 1000000);
}
```

The Rust compiler and KLEE are in the Dockerfile so start the Docker image
by running

``` shell
docker/run
cd demos/simple/klee
```

All remaining commands in this file will be run in this docker
image.

(It is usually easiest to run this in one terminal while using
a separate editor to edit the files in another terminal.)


## Compiling Rust for verification

The [Rust compiler works][MIR blog post]
in four stages:
first, it compiles Rust source code to HIR (the high-level IR);
HIR is converted to MIR (the mid-level IR);
MIR is converted to LLVM IR (the low-level virtual machine IR);
and, finally, it compiles LLVM IR down to machine code and links the result.

When using an LLVM-based tool like KLEE, we need to modify this behaviour
so that the LLVM IR is linked and saved to a file.
This can be done by passing extra flags to `rustc` when using `cargo`
to instruct `rustc` to link the bitcode files.

```
export RUSTFLAGS="-Clto -Cembed-bitcode=yes --emit=llvm-bc $RUSTFLAGS"
```

We also have to pass some configuration flags to `cargo` and `rustc` to
configure the `verification-annotations` crate correctly.

```
export RUSTFLAGS="--cfg=verify $RUSTFLAGS"
```

Our goal is to find bugs so we turn on some additional error checking
in the compiled code to detect arithmetic overflows.

```
export RUSTFLAGS="-Warithmetic-overflow -Coverflow-checks=yes $RUSTFLAGS"
```

And, when we find a bug, we want to report it as efficiently as
possible so we make sure that the program will abort if it panics.


```
export RUSTFLAGS="-Zpanic_abort_tests -Cpanic=abort $RUSTFLAGS"
```

With all those definitions, we can now run

```
cargo build --features=verifier-klee
```

Depending on which platform you are running on, the resulting LLVM bitcode file
may be placed placed in `target/debug/deps/try-klee.bc` (OSX) or in a file with
a name like `target/debug/deps/try_klee-6136b0f50ac42b91.bc` (Linux).
At least for simple cases, we can refer to both files as
`target/debug/deps/try_klee*.bc` and not worry about the exact filename.

The above should have worked. But, if it produced a linking error involving
`-lkleeRuntest` you may have to point the linker at the directory where the KLEE
library was installed.  For example, on OSX you might have installed it in
`$HOME/homebrew/lib` and need to use this command

```
export RUSTFLAGS="-L$HOME/homebrew/lib -C link-args=-Wl,-rpath,$HOME/homebrew/lib $RUSTFLAGS"
```


### Compiling large programs

Finally, on larger, more complex projects than this example, we have seen
problems with the Rust compiler generating SSE instructions that KLEE does not
support.
We don't have a complete solution for this but we have found that it helps
to compile with a low (but non-zero) level of optimization, to disable
more sophisticated optimizations and to disable SSE and AVX features.

```
export RUSTFLAGS="-Copt-level=1 $RUSTFLAGS"
export RUSTFLAGS="-Cno-vectorize-loops -Cno-vectorize-slp $RUSTFLAGS"
export RUSTFLAGS="-Ctarget-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2 $RUSTFLAGS"
```

(This is not needed for our simple example.)


## Running KLEE

Having built a bitcode file containing the program and all the libraries that
it depends on, we can now run KLEE like this:

```
klee --libc=klee --silent-klee-assume --output-dir=kleeout --warnings-only-to-file target/debug/deps/try_klee*.bc
```

This command will produce output like this

```
KLEE: output directory is "try-klee/kleeout"
KLEE: Using STP solver backend
... (possibly warnings about different target triples - probably benign)
KLEE: done: total instructions = 10153
KLEE: done: completed paths = 3
KLEE: done: generated tests = 1
```

(On OSX, it may also crash and produce a stack dump _after_ producing that output?)

This shows that KLEE explored three paths through the above code,
found one path that failed and generated
one file containing inputs that can trigger that failing path.
To find those input values, we look in the directory `kleeout`
for files with names like `test000001.ktest`.
These are binary files that can be examined using KLEE's `ktest-tool`

```
$ ktest-tool kleeout/test000001.ktest
ktest file : 'kleeout/test000001.ktest'
args       : ['target/debug/deps/try_klee.bc']
num objects: 2

object 0: name: 'unnamed'
object 0: size: 4
object 0: data: b'\x01\x00\x00\x00'
object 0: hex : 0x01000000
object 0: int : 1
object 0: uint: 1
object 0: text: ....

object 1: name: 'unnamed'
object 1: size: 4
object 1: data: b'\x01\x00\x00\x00'
object 1: hex : 0x01000000
object 1: int : 1
object 1: uint: 1
object 1: text: ....
```

This shows that the program created two non-deterministic objects each called
'unnamed` and each of size 4.
KLEE does not know _the_ way to interpret those so it displays for them
several common interpretations.


## Replaying the input values

When we compiled the program to produce LLVM bitcode, we also generated
a conventional binary.
We can run this in the normal way:

```
cargo run --features=verifier-klee
```

and KLEE will prompt us for the name of the input file

```
KLEE-RUNTIME: KTEST_FILE not set, please enter .ktest path:
```

Or, we can specify the input file when we invoke KLEE:

```
$ KTEST_FILE=kleeout/test000001.ktest cargo run --features=verifier-klee
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/try-klee`
Test values: a = 1, b = 1
```

This provides an easier way to view the test values chosen: using Rust's
builtin print function.

## Handling larger programs

The instructions above work fine for small programs.
But, as we tackle programs with dependencies, we need to add a few more
flags to make it all work.

### Compiling larger programs

If you add the following dependency on `serde`

```
serde = { version = "1.0", features = ["derive"] }
```

you will likely get a linking error

```
error: cannot prefer dynamic linking when performing LTO
```

The
([incredibly](https://github.com/rust-lang/cargo/issues/6375#issuecomment-444900324)
[obscure](https://github.com/rust-lang/cargo/issues/7539))
workaround for this is to specify a target explicitly.
(I have no idea why this helps!)

The first step is to figure out what target you are currently using.
You want the "default host" reported by "rustup show"

```
$ rustup show | grep Default
Default host: x86_64-unknown-linux-gnu
```

Now that we know the target, we add `--target` to the `cargo build` command

```
cargo build --features=verifier-klee --target=x86_64-unknown-linux-gnu
```

Unfortunately, this changes where the final bitcode file is put.
Instead of
`target/debug/deps/try_klee*.bc`
it is put in
`target/x86_64-unknown-linux-gnu/debug/deps/try_klee*.bc`.


### Using KLEE with larger programs

In the above, we used this command to invoke KLEE

```
klee --libc=klee --output-dir=kleeout --warnings-only-to-file target/debug/deps/try_klee*.bc
```

Some additional flags that are worth using for larger examples are

- `--exit-on-error` causes KLEE to exit as soon as it finds a problem.
  (KLEE's default mode is to keep searching for more problems.)

- `--disable-verify` works around a problem in KLEE 
  caused by [an interaction between debug information and inlining](https://github.com/klee/klee/issues/937).

These flags are all added _before_ the bitcode file like this

```
klee --output-dir=kleeout --warnings-only-to-file --exit-on-error \
     --libc=klee --silent-klee-assume --disable-verify \
     target/x86_64-unknown-linux-gnu/debug/deps/try_klee*.bc
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
[demos/simple/klee/src/main.rs]:  {{site.gitrepo}}blob/main/demos/simple/klee/src/main.rs
[demos/simple/klee/verify.sh]:    {{site.gitrepo}}blob/main/demos/simple/klee/verify.sh
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

