---
layout: post
title: "Using Seahorn"
permalink: /using-seahorn/
---

![SeaHorn logo](https://seahorn.github.io/images/seahorn-logo.png){: style="float: left; width: 10%; padding: 1%"}
[SeaHorn] is an automated analysis framework for [LLVM]-based languages.
For users [SeaHorn] provides a push-button verification tool, and for
researchers its modular design provides an extensible and customizable
framework for experimenting with new software verification techniques.
In contrast to [KLEE] (see previous [post][Using KLEE]) which uses symbolic
execution, [SeaHorn] uses software model checking and abstract interpretation.
This, we hope, will provide better results in some cases.

In this post we will do a walk-through example of using [SeaHorn] to verify a
Rust program.
We will:
1. Compile the program to generate bitcode (LLVM IR);
1. Run rvt-patch-llvm on the bitcode;
1. Run SeaHorn on the patched bitcode; and
1. Run SeaHorn again to produce a trace of the bug.

(This post borrows heavily from the [Using KLEE] post)

_Note:
The recommended way to use [SeaHorn] is with the `propverify` library and the
`cargo-verify` tool as described [here][Using PropVerify].
This note describes how to use the [SeaHorn] directly in case you wonder how
`cargo-verify` works or want to add support for a different tool.
This is going to be a fairly low-level description and most people will be
happier not knowing how the sausage is made.
(Since there are so many low-level details, there is a risk that we will forget
to update this document so you may have to [use the source][cargo-verify source].)_

## A small test program

This code is in [demos/simple/seahorn/src/main.rs] and the shell commands in this
file are in [demos/simple/seahorn/verify.sh].

``` rust
use verification_annotations::prelude::*;

fn main() {
    let a = u32::abstract_value();
    let b = u32::abstract_value();
    verifier::assume(1 <= a && a <= 1000);
    verifier::assume(1 <= b && b <= 1000);
    let r = a*b;
    verifier::assert!(1 <= r && r < 1000000);
}
```

The Rust compiler, [SeaHorn], and everything else we need is already installed
in the Docker image provided in the [RVT git repo], so start a Docker container
from the image by running:

``` shell
docker/run
# You should now be inside the Docker container
cd demos/simple/seahorn
```

All remaining commands in this file will be run in this docker container.

(It is usually easiest to run this in one terminal while using
another terminal, or a separate editor, to edit the files.)

## Compiling Rust for verification

The [Rust compiler works][MIR blog post]
in four stages:
first, it compiles Rust source code to HIR (the high-level IR);
HIR is converted to MIR (the mid-level IR);
MIR is converted to LLVM IR (the low-level virtual machine IR);
and, finally, it compiles LLVM IR down to machine code and links the result.

When using an LLVM-based tool like [SeaHorn], we need to modify this behaviour
so that the LLVM IR (instead of machine code) is linked and saved to a file.
This can be done by passing extra flags to `rustc`, instructing it to link the
bitcode files.
When using `cargo` these flags are passed through the `RUSTFLAGS` environment
variable.

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
cargo build --features=verifier-seahorn
```

Depending on which platform you are running on, the resulting LLVM bitcode file
may be placed placed in `target/debug/deps/try_seahorn.bc` (OSX) or in a file with
a name like `target/debug/deps/try_seahorn-*.bc` (Linux).
At least for simple cases, we can refer to both files as
`target/debug/deps/try_seahorn*.bc` and not worry about the exact filename.


<!-- ### Compiling large programs -->

<!-- Finally, on larger, more complex projects than this example, we have seen -->
<!-- problems with the Rust compiler generating SSE instructions that KLEE does not -->
<!-- support. -->
<!-- We don't have a complete solution for this but we have found that it helps -->
<!-- to compile with a low (but non-zero) level of optimization, to disable -->
<!-- more sophisticated optimizations and to disable SSE and AVX features. -->

<!-- ``` -->
<!-- export RUSTFLAGS="-Copt-level=1 $RUSTFLAGS" -->
<!-- export RUSTFLAGS="-Cno-vectorize-loops -Cno-vectorize-slp $RUSTFLAGS" -->
<!-- export RUSTFLAGS="-Ctarget-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2 $RUSTFLAGS" -->
<!-- ``` -->

<!-- (This is not needed for our simple example.) -->


## Running SeaHorn

Having built a bitcode file containing the program and all the libraries that it
depends on, we now need to fix a few things in the bitcode file to make it
suitable for [SeaHorn]:
* Delete the `main` function rustc added.
In the LLVM IR code that the Rust compiler generates the `main` function from
main.rs is renamed to something like
`_ZN11try_seahorn4main17h8733453d83f64f5aE`.
The entry point to the LLVM IR code is a new `main` function that does some
initialisation and then calls the original main function.
This initialisation is not handled very well by [SeaHorn] at the moment, so we
just delete the `main` function and tell [SeaHorn] the entry point is the
original main function with the mangled name.
Note that it is not enough to just tell [SeaHorn] to use the mangled name as the
entry point, we also need to delete `main`, otherwise [SeaHorn] gets confused.
* Remove the body of `_eprint` and `_print`.
Those two functions are the internal implementations of Rust's std printing
macros.
[SeaHorn] can't handle those functions at the moment.
* Replace panic handling.
Instead of the regular panic handling we call `__VERIFIER_error` which [SeaHorn]
will report as an error (if reachable).

The `rvt-patch-llvm` tool can do all that for us:

``` shell
rvt-patch-llvm -o try_seahorn.patch.bc --seahorn -vv target/debug/deps/try_seahorn-*.bc
```

The bitcode file `try_seahorn.patch.bc` is now ready to be processed by
[SeaHorn].

To run [SeaHorn] we use a configuration file that we checkout from
[verify-c-common].
The Docker image is configured so that `SEAHORN_VERIFY_C_COMMON_DIR` points to
that checkout.
In addition, because we deleted the `main` function rustc added to the bitcode
(which does some runtime initialisation and then calls the original main
function) we have to tell [SeaHorn] the name of the entry function.
To find this name execute this:

``` shell
llvm-nm --defined-only try_seahorn.patch.bc | grep main | cut -d ' ' -f3
```

The output should look something like this:

``` shell
_ZN11try_seahorn4main17h8733453d83f64f5aE
```

(the suffix after "17h" might be different)

We can now put everything together and run [SeaHorn] like this:

``` shell
sea yama -y ${SEAHORN_VERIFY_C_COMMON_DIR}/seahorn/sea_base.yaml bpf --entry="_ZN11try_seahorn4main17h8733453d83f64f5aE" try_seahorn.patch.bc
```

This command will produce a long output that should end like this:

```
sat


************** BRUNCH STATS ***************** 
[...]
************** BRUNCH STATS END ***************** 
```

The "sat" indicates that [SeaHorn] was able to find an execution of the program
that violates the assertion (or panics).
[SeaHorn] can produce a trace of the execution like this:

``` shell
sea yama -y ${SEAHORN_VERIFY_C_COMMON_DIR}/seahorn/sea_cex_base.yaml bpf --entry="_ZN11try_seahorn4main17h8733453d83f64f5aE" try_seahorn.patch.bc
```

As [SeaHorn] is an LLVM-based tool, the trace is an LLVM IR trace.
Moreover, [SeaHorn] does a few transformations and optimisations to the program,
and the trace is over the resulting program.
Fortunately, [SeaHorn] preserves some debug information that allows it to print
source code line-numbers along the trace (in Cyan).

The end of the trace is usually code from rust/src/libcore/fmt/mod.rs that
handles the formatting for a print function that reports the assertion that
failed.
Scroll up the trace until you find the first line-number that is from main.rs.
This should be the line where the assertion that failed is.
In this case it is "[src/main.rs:10]".
To find the value of `r` that caused the assertion to fail, keep scrolling up
until you find "[src/main.rs:9]" (the line where `r` is assigned its value):

``` shell
  %_18.0.i.i = extractvalue { i32, i1 } %_9, 0, !dbg !180
  %_18.0.i.i (0xf4240:bv(32)) [src/main.rs:9]
  r = _18.0.i.i (0xf4240:bv(32))
```

This violates the `r < 1000000` part of the assertion (0xf4240 is 1000000).
If you keep scrolling up you can also find the values that were assigned to `a`
and `b`:

``` shell
  [...]
  %_5 = call i32 @__VERIFIER_nondet_u32() #12, !dbg !156
  %_5 (1000:bv(32)) [/home/rust-verification-tools/verification-annotations/src/seahorn.rs:92]
  a = _5 (1000:bv(32))
enter: abstract_value<u32>
enter: default
  self = i32 0 0
  %_6 = call i32 @__VERIFIER_nondet_u32() #12, !dbg !164
  %_6 (1000:bv(32)) [/home/rust-verification-tools/verification-annotations/src/seahorn.rs:92]
  b = _6 (1000:bv(32))
  [...]
```

Unfortunately, [SeaHorn] reports a line number from the verification-annotations
crate for those assignments.
In general, you can look for calls to `__VERIFIER_nondet_<type>` in the trace to
find the concrete values [SeaHorn] picked for
`verifer_nondet`/`abstract_value`/`symbolic`.

## Handling larger programs

The `-Clto` flag we pass to the Rust compiler conflicts with some default
configurations of the compiler.
If you add the following dependency on `serde` to `Cargo.toml`

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
cargo build --features=verifier-seahorn --target=x86_64-unknown-linux-gnu
```

This changes where the bitcode file is put.
Instead of
`target/debug/deps/try_seahorn*.bc`
it is put in
`target/x86_64-unknown-linux-gnu/debug/deps/try_seahorn*.bc`.

From here the process is the same.


[CC-rs crate]:                    https://github.com/alexcrichton/cc-rs/
[Cargo build scripts]:            https://doc.rust-lang.org/cargo/reference/build-scripts.html
[Clang]:                          https://clang.llvm.org/
[Crux-MIR]:                       https://github.com/GaloisInc/mir-verifier/
[Docker]:                         https://www.docker.com/
[GraalVM and Rust]:               https://michaelbh.com/blog/graalvm-and-rust-1/
[Hypothesis]:                     https://hypothesis.works/
[KLEE]:                           https://klee.github.io/
[LLVM]:                           https://llvm.org/
[Linux driver verification]:      http://linuxtesting.org/ldv/
[MIR blog post]:                  https://blog.rust-lang.org/2016/04/19/MIR.html
[PropTest book]:                  https://altsysrq.github.io/proptest-book/intro.html
[PropTest]:                       https://github.com/AltSysrq/proptest/
[Rust benchmarks]:                https://github.com/soarlab/rust-benchmarks/
[Rust port of QuickCheck]:        https://github.com/burntsushi/quickcheck/
[Rust's runtime]:                 https://blog.mgattozzi.dev/rusts-runtime/
[SMACK]:                          https://smackers.github.io/
[SV-COMP]:                        https://sv-comp.sosy-lab.org/2020/rules.php
[SeaHorn]:                        https://github.com/seahorn/seahorn
[dev10]:                          https://github.com/seahorn/seahorn/tree/dev10
[rustc]:                          https://doc.rust-lang.org/rustc/
[std::env::args source code]:     https://github.com/rust-lang/rust/blob/master/library/std/src/sys/unix/args.rs
[verify-c-common]:                https://github.com/seahorn/verify-c-common

[RVT git repo]:                   {{site.gitrepo}}/
[cargo-verify source]:            {{site.gitrepo}}blob/main/cargo-verify/
[demos/simple/klee/src/main.rs]:  {{site.gitrepo}}blob/main/demos/simple/klee/src/main.rs
[demos/simple/klee/verify.sh]:    {{site.gitrepo}}blob/main/demos/simple/klee/verify.sh
[demos/simple/seahorn/src/main.rs]:{{site.gitrepo}}blob/main/demos/simple/seahorn/src/main.rs
[demos/simple/seahorn/verify.sh]: {{site.gitrepo}}blob/main/demos/simple/seahorn/verify.sh
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

