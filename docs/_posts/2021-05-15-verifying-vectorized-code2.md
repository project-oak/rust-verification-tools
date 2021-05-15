---
layout: post
title: Verifying vectorized Rust revisited
---

![Rust logo](https://www.rust-lang.org/static/images/rust-logo-blk.svg){: style="float: left; width: 10%; padding: 1%"}
Research is characterized by allowing yourself to make mistakes: performing
experiments; drawing conclusions; later, realizing that your
experiment was not sufficient and you got it wrong; and
trying again.
Back in March, we thought that we knew [how to deal with vectorized Rust][Hand
vectorized Rust]: tell the compiler not to auto-vectorize code; tell the compiler
not to use vector instructions; and use existing conditional compilation
feature flags to disable hand-vectorized code.
Unfortunately, two of those three ideas don't work – but we think we have a
viable approach now.

The first of these ideas probably works: you can tell the compiler not to
auto-vectorize code. If you are using cargo, you can control this with
`RUSTFLAGS`; if you are using rustc directly, you can directly disable
auto-vectorization when invoking rustc.  And, if your program calls C code (or
uses a crate that calls C code) and uses the [CC-rs crate] to compile the code,
then you can easily pass flags to the C compiler to disable auto-vectorization.
(If your program calls C code that was not compiled using the CC-rs crate, you
will need to modify the C code's  build system – which is a bit more work.)

The reason that telling the compiler not to use vector instructions does not
work is that it is not possible to turn off the x86 architecture's SSE2
instructions without breaking floating point. We recently realized that the
`-Ctarget-feature=-sse2` flag turns of both SSE2 vector instructions and
also support for IEEE floating point.
With IEEE floating point disabled, LLVM attempts to use the old 80-bit x87 floating point unit and then
fails an assertion while compiling your code.
In short, you cannot use the `-sse2` flag with Rust.
It took us a while to recognize that we could not just disable SSE2 because,
the technique we were using to disable SSE2 in the standard library turned
out to do nothing. 
We thought that we could do this using the environment variable `RUSTFLAGS_STAGE_NOT_0`
to disable SSE2 (and other x86 vector extensions) when compiling the
standard library.
Alas, this environment variable has been renamed so our attempt to disable SSE2
was being ignored.

Finally, the reason that using existing conditional compilation feature flags
to disable hand-vectorized code does not work is that there is no single feature flag that
everybody uses.
In some crates, the [miri] flag is used to disable vectorization but, as you might
imagine, the miri flag has other effects.
In other crates, there is an explicit way of disabling vectorization using
some crate-specific feature flag.
And, in other crates, there is no way to disable vectorization.
It might be a good idea to have a standard feature flag to disable
hand-vectorized code but, at least for now,
attempts to disable hand-vectorized code require different approaches for
each crate you want to use.


## Emulate, don't eliminate

Before I describe our new solution to this problem, it's worth asking whether
this matters?  If your approach to verification is to use function contracts to
limit the scope of your work to the function you are currently working on, then
it will not matter at all.  On the other hand, if your goal is like ours of
verifying entire programs without the overhead of specifying every function,
then this will probably matter a lot because, even if the code you wrote does
not use vector instructions, the chances are high that your code depends on
a crate that depends on a crate that uses one of `regex/aho-corasick`,
`hashbrown` or `std::collections::HashMap` – all of which use vector
instructions.  So, if you are interested in verifying entire programs, you
probably need a way of handling vector instructions.  Alas, no Rust
verification tools that we know of actually support vector instructions: they
just fail with a message about unsupported instructions.

Our new approach to handling vector instructions is to *emulate* vector
instructions instead of trying to *eliminate* them.  That is we need a [SIMD
emulation library] and then we need to arrange for verification tools to use
that emulation library when they encounter vector instructions instead of
reporting that they have found an unsupported instruction.

Our [SIMD emulation library] implements the processor-specific SIMD intrinsics
that we have been finding in the LLVM bitcode generated from Rust programs.
(This is probably a subset of the intrinsics that you would need if your
verification tools are based on MIR. We would happily add additional intrinsics
if other verification tools need them.)


## Implementing the SIMD emulation library

SIMD instruction sets are typically quite large: there are a lot of
instructions to support but there three features of SIMD instructions that reduce
the effort required to write an emulation library.

1. Almost all SIMD instructions fit into one of three patterns: map-like
   instructions such as vector addition that process vector elements
   independently of each other; fold-like instructions that combine vector
   elements to give either a shorter vector or a scalar value; and permutation
   instructions that rearrange vector elements.

2. Almost all SIMD instructions are based on taking a large register and
   dividing it into a number of elements of with 8, 16, 32 or 64 bits
   and, with the exception of Arm's SVE, the register size and the number of
   elements is a fixed power of two.

3. Almost all SIMD instructions have either two vector arguments or a vector argument and a scalar argument.

These observations allow us to make very effective use of Rust's macros and
Rust's `Fn` trait when writing our emulation library.

For example, the x86 architecture has a family of instructions called PSRLI
that combine a vector with a scalar immediate value.
Each element of the vector argument is shifted by the distance specified by the
scalar argument.

The action on each vector element can be described by a scalar function
that shifts a scalar value by a scalar shift amount.
For example, for 32-bit elements, the function looks like this.

``` rust
/// Logical shift right by 8-bit immediate (0 if shift distance too large)
pub fn srl_immed_u32_u8(x: u32, imm8: u8) -> u32 {
    if imm8 > 31 {
        0
    } else {
        x >> imm8
    }
}
```

To implement the corresponding vector function, we "lift" the scalar function
so that it operates on vectors instead.
Since this pattern is very common, we do this by defining a
lifting function such as this one that operates on vectors of four elements
to produce a function with one vector argument and one scalar argument.

``` rust
// lift a binary operation over vector and scalar
pub fn lift4_vs_v<F, A, B, R>(f: F, a: A::Vec, b: B) -> R::Vec
where
    F: Fn(A, B) -> R,
    A: Vector4,
    B: Copy,
    R: Vector4,
{
    let r0 = f(A::get0(&a), b);
    let r1 = f(A::get1(&a), b);
    let r2 = f(A::get2(&a), b);
    let r3 = f(A::get3(&a), b);
    R::new(r0, r1, r2, r3)
}
```

It is now easy to combine these functions to emulate the 32-bit version of the
PSRLI instruction.


```
#[no_mangle]
unsafe extern "C" fn llvm_x86_sse2_psrli_d(a: u32x4, imm8: i32) -> u32x4 {
    lift4_vs_v(scalar::srl_immed_u32_u8, a, imm8 as u8)
}
```

Note that the type `u32x4` is the Rust type representing a vector of four
32-bit values.
This type implements the trait `Vector4` used in the definition of
`lift4_vs_v`.

And, by defining traits `Vector2`, `Vector4`, `Vector8`, `Vector16` and
`Vector32`, and associated lifting functions, we can very quickly implement
other versions of the SSE2 PSRLI instruction.


``` rust
#[no_mangle]
unsafe extern "C" fn llvm_x86_sse2_psrli_b(a: u8x16, imm8: i32) -> u8x16 {
    lift16_vs_v(scalar::srl_immed_u8_u8, a, imm8 as u8)
}

#[no_mangle]
unsafe extern "C" fn llvm_x86_sse2_psrli_w(a: u16x8, imm8: i32) -> u16x8 {
    lift8_vs_v(scalar::srl_immed_u16_u8, a, imm8 as u8)
}

#[no_mangle]
unsafe extern "C" fn llvm_x86_sse2_psrli_q(a: u64x2, imm8: i32) -> u64x2 {
    lift2_vs_v(scalar::srl_immed_u64_u8, a, imm8 as u8)
}
```


## Using the SIMD emulation library

There is no compiler flag that will cause rustc or LLVM to use our SIMD
emulation library so whether our verification tool uses MIR or LLVM IR,
the output of the compiler will contain calls to the official SIMD intrinsics
instead of the SIMD emulation library.

One option for using the SIMD emulation library would be to modify our
verification tool to recognize calls to SIMD intrinsics and, instead,
to treat them as calls to the emulation functions.

But, one of our project goals is to be able to use as many different
verification tools as possible and we did not want to have to modify multiple
tools.
So, instead, we extended the post-processor `rvt-patch-llvm` that we wrote
[to handle initializers and command-line arguments][Using ARGV]
to replace all calls to SIMD intrinsics with calls to our SIMD
emulation library. (The code that does the patching is [here][SIMD patching tool].)


## Summary

Handling processor-specific vector intrinsics was harder than we originally
thought.

Although it initially seemed to be effective, we realized that our approach of
trying to *eliminate* vector intrinsics was not working.  This forced us to
"bite the bullet" and write a partial SIMD emulation library.  This turned out
to be easier than we had feared because, although SIMD instruction sets are
huge, they contain a large amount of regularity.

Our emulation library meets our needs but we believe that it would also be
useful to teams developing other Rust verification tools.  We would be very
happy to work with other Rust verification teams to create a single SIMD
emulation library that meets everybody's needs.



[Hand vectorized Rust]:           {{site.baseurl}}{% post_url 2021-03-20-verifying-vectorized-code %}
[Using FFI]:                      {{site.baseurl}}{% post_url 2020-12-11-using-ffi %}
[Using ARGV]:                     {{site.baseurl}}{% post_url 2020-09-09-using-argv %}

[SIMD emulation library]:         https://github.com/project-oak/rust-verification-tools/tree/main/simd_emulation
[SIMD patching tool]:             https://github.com/project-oak/rust-verification-tools/blob/7277d4c9a156f1824adb597f323a02eebb6d55e3/rvt-patch-llvm/src/main.rs#L109

[CC-rs crate]:                    https://github.com/alexcrichton/cc-rs/
[miri]:                           https://github.com/rust-lang/miri
