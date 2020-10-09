# Using KLEE

_Note:
The recommended way to use KLEE is with the `propverify` library
and the `cargo-verify` script
as described [here](using-propverify.md).
This note describes how to use the KLEE directly
in case you wonder how `cargo-verify` works or want
to add support for a different tool.6
This is going to be a fairly low-level description and most
people will be happier not knowing how the sausage is made.
(Since there are so many low-level details, there is a risk
that we will forget to update this document so you may have to [use the
source](../scripts/cargo-verify))._

The Rust compiler is based on the LLVM platform and
KLEE is based on LLVM so the steps involved in using
KLEE to verify Rust programs are:

1. Compile the program to generate LLVM bitcode.
2. Run KLEE on the bitcode file.
3. Examine the output of KLEE.


## A small test program

As a running example, we will use the same example
that we used to explain [how to use the verification-annotations
crate](using-annotations.md).
(Except that we will fix the deliberate bug.)


```
use verification_annotations as verifier;

fn main() {
    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(1 <= a && a <= 1000);
    verifier::assume(1 <= b && b <= 1000);
    if verifier::is_replay() {
        eprintln!("Test values: a = {}, b = {}", a, b);
    }
    let r = a*b;
    verifier::assert!(1 <= r && r <= 1000000);
}
```

and we will create a crate in which to experiment with this
code

```
cargo new try-klee
cd try-klee

cat >> Cargo.toml  << "EOF"

verification-annotations = { path="../verification-annotations" }

[features]
verifier-klee = ["verification-annotations/verifier-klee"]
EOF

cat > src/main.rs  << "EOF"
use verification_annotations as verifier;

fn main() {
    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(1 <= a && a <= 1000);
    verifier::assume(1 <= b && b <= 1000);
    if verifier::is_replay() {
        eprintln!("Test values: a = {}, b = {}", a, b);
    }
    let r = a*b;
    verifier::assert!(1 <= r && r <= 1000000);
}
EOF
```


## Compiling Rust for verification

The [Rust compiler works](https://blog.rust-lang.org/2016/04/19/MIR.html)
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
RUSTFLAGS="-Copt-level=1 $RUSTFLAGS"
RUSTFLAGS="-Cno-vectorize-loops -Cno-vectorize-slp $RUSTFLAGS"
RUSTFLAGS="-Ctarget-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2 $RUSTFLAGS"
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
klee --output-dir=kleeout --warnings-only-to-file target/debug/deps/try_klee*.bc
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
