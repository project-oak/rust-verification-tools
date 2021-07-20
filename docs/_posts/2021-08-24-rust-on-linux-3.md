---
layout: post
title: Using KLEE on Rust-for-Linux (part 3)
---

![Linux logo](https://cdn.kernel.org/pub/linux/kernel/v2.0/Logo.gif){: style="float: left; width: 10%; padding: 1%"}
To conclude this series on using [KLEE] with [Rust for Linux],
we are going to write a simple verification harness
for testing the `rust_semaphore` device driver written in Rust
and symbolically execute the verification harness in [KLEE].
It's worth repeating that the goal of these posts is not to
verify the Rust for Linux code or to find bugs in it.
Instead, my goal is that you should be able to apply
these ideas to do that verification yourself.
You might choose to continue in the same direction as these posts suggest
by creating better mocks and test harnesses.
Or you might want to try something else based on different answers
to the questions in the [first part of this series][Using KLEE with Rust for Linux (part 1)];
or, perhaps, using some tool other than [KLEE]
that I described in the [second part of the series][Using KLEE with Rust for Linux (part 2)].
The code changes described in this post are
in [this branch of my fork of Rust for Linux](https://github.com/alastairreid/linux/tree/klee).


## Beyond concrete tests

Recall from the previous part of this series that we wrote a test harness like
this

``` rust
pub fn test_fileops() -> Result<()> {
    let registration = &RustSemaphore::init()?._dev;
    pr_info!("Initialized");
    let file_state = *mk_file_state::<Arc<Semaphore>, FileState>(registration)?;
    pr_info!("Got filestate");
    let file = File::make_fake_file();
    test_write(&file_state, &file, 128);
    test_read(&file_state, &file, 128);
    Ok(())
}
```

This is a fairly limited test.
It only tries one sequence of operations and only tries reads and writes of size 128.

We could improve this by creating several tests that try different sequences
of operations and that try various different sizes.
Or, we could improve it by using fuzz-testing: trying several different
*random* sequences of operations where each operation reads or
writes a *random* amount of data.
Or, we could use the [KLEE] symbolic execution tool to check all sequences
up to some length and all possible values.
Let's try the latter.


## Using the verification-annotations library

Our preferred way of using [KLEE] is to use our [PropVerify library][Using PropVerify] since
it is designed to work with multiple formal verification tools.
Also, since PropVerify implements the same API as [PropTest], you can switch
back and forth between verification tools and fuzz-testing just by
choosing a different command-line option: no need to change the
verification/fuzzing harness.

In principle, PropVerify should work just fine with the Rust for Linux code.
But, we're used to using the PropVerify crate with cargo and
it is a bit tedious to figure out how to survive without cargo.

So, if you have been following this series so far, you'll be unsurprised to
learn that I am going to take a shortcut in the interest of making progress
quickly.
I will use the simpler, lower-level [verification-annotations][Using
verification-annotations] library that PropVerify is built on.
This still provides theoretical[^theoretical-portability]
portability across different verification tools
but it does not support fuzzing.

[^theoretical-portability]:
    Although the verification-annotations library does work with several
    different verification tools, as far as I know, KLEE is the only
    tool that can cope with something as large and complex as
    Rust for Linux at the moment.
    This will probably change in the future though so, someday,
    this *theoretical* portability will become real.

To use the verification-annotations crate, we first need to add
it to the Linux repo[^no-std].
The easy way to do this is to create a symbolic link
to the crate.

[^no-std]:
    I also had to I had to add `#[cfg(std)]` annotations throughout the
    verification-annotations codebase since the Linux kernel is built in a no-std
    environment.
    The PropTest library also supports no-std but we have not yet updated PropVerify.

``` shell
ln -s $RVT_DIR/verification-annotations/src rust/verification-annotations
```

And modify the Makefile to use the crate and use the "verifier-klee"
feature.

```
diff --git a/scripts/Makefile.build b/scripts/Makefile.build
index ed8011d5b30b..4e9f281e4bba 100644
--- a/scripts/Makefile.build
+++ b/scripts/Makefile.build
@@ -300,6 +300,7 @@ quiet_cmd_rustc_o_rs = $(RUSTC_OR_CLIPPY_QUIET) $(quiet_modtag) $@
        RUST_MODFILE=$(modfile) \
        $(RUSTC_OR_CLIPPY) $(rustc_flags) $(rustc_cross_flags) \
                --extern alloc --extern kernel \
+               --extern verification-annotations --cfg 'feature="verifier-klee"' \
                --crate-type rlib --out-dir $(obj) -L $(objtree)/rust/ \
                --crate-name $(patsubst %.o,%,$(notdir $@)) $<; \
        mv $(obj)/$(subst .o,,$(notdir $@)).d $(depfile); \

diff --git a/Makefile b/Makefile
index ff9e419b4420..3c3d4529a7dd 100644
--- a/Makefile
+++ b/Makefile
@@ -515,6 +515,7 @@ KBUILD_RUSTC_TARGET := $(srctree)/arch/$(SRCARCH)/rust/target.json
 KBUILD_RUSTCFLAGS := --emit=dep-info,obj,metadata --edition=2018 \
                     -Cpanic=abort -Cembed-bitcode=n -Clto=n -Crpath=n \
                     -Cforce-unwind-tables=n -Ccodegen-units=1 \
+                     --cfg 'feature="verifier-klee"' \
                     -Zbinary_dep_depinfo=y -Zsymbol-mangling-version=v0
 KBUILD_AFLAGS_KERNEL :=
 KBUILD_CFLAGS_KERNEL :=
@@ -526,6 +527,7 @@ KBUILD_LDFLAGS_MODULE :=
 KBUILD_LDFLAGS :=

diff --git a/rust/Makefile b/rust/Makefile
index b1aea0db415f..13670eb1e040 100644
--- a/rust/Makefile
+++ b/rust/Makefile
@@ -7,6 +7,7 @@ extra-$(CONFIG_RUST) += libmodule.so

 extra-$(CONFIG_RUST) += bindings_generated.rs
 obj-$(CONFIG_RUST) += alloc.o kernel.o
+obj-$(CONFIG_RUST) += verification_annotations.o
 obj-$(CONFIG_RUST) += rstubs.o
 obj-$(CONFIG_RUST) += stubs.o
 extra-$(CONFIG_RUST) += exports_alloc_generated.h exports_kernel_generated.h
@@ -176,3 +177,11 @@ $(objtree)/rust/kernel.o: $(srctree)/rust/kernel/lib.rs $(objtree)/rust/alloc.o
     $(objtree)/rust/build_error.o \
     $(objtree)/rust/libmodule.so $(objtree)/rust/bindings_generated.rs FORCE
        $(call if_changed_dep,rustc_library)
+
+$(objtree)/rust/verification_annotations.o: private rustc_target_flags = --extern alloc \
+    --extern build_error \
+    --extern module=$(objtree)/rust/libmodule.so
+$(objtree)/rust/verification_annotations.o: $(srctree)/rust/verification_annotations/lib.rs $(objtree)/rust/alloc.o \
+    $(objtree)/rust/build_error.o \
+    $(objtree)/rust/libmodule.so $(objtree)/rust/bindings_generated.rs FORCE
+       $(call if_changed_dep,rustc_library)
```

And now we can compile, link and run the rust-semaphore test from before

``` shell
make CC=wllvm -j16 -k
llvm-link-11 samples/rust/rust_semaphore.bc samples/rust/.rust_semaphore.mod.o.bc rust/alloc.bc rust/kernel.bc rust/core.bc rust/rstubs.bc rust/.stubs.o.bc rust/verification_annotations.bc -o t.bc
klee --entry-point=test_fileops t.bc
```

Since we are invoking the same fixed test as before, it will not do anything
different -- we are just checking that things still work.


## A simple verification harness: Arbitrary sequences of operations

Let's take the test above and make it just a little more flexible: instead of
performing a write followed by a read, let's explore all possible sequences of
reads and writes (up to some limit on the length of the sequence).

To do this, we will generate a sequence of "symbolic" choices about what
operation to perform next.
We do this using the verification-annotations API to request KLEE to generate
symbolic values.
Symbolic values are like variables "x" and "y" in math expressions like "x\*x + y\*y <= 100":
they can take on any legal concrete value for their type like 42 or 0x8000_0000.
By using symbolic values, we are able to check whether the program could fail
for *any* possible concrete values.

The function used to create symbolic values is `abstract_value` that takes no
arguments and returns a value. When run in KLEE, this creates a symbolic value.

``` rust
pub trait AbstractValue: Sized {
    fn abstract_value() -> Self;
}
```

To make a sequence of choices of length `n`, we use a for loop,
and match on a symbolic value to select one of several choices.
We use `verifier::reject()` to discard symbolic values that
don't match any choice.

``` rust
for _ in 0..n {
    // make arbitrary choice of what to do next
    match AbstractValue::abstract_value() {
        0 => ...,
        1 => ...,
        ...,
        _ => verifier::reject() // ignore any other values
    }
}
```

The choices that we want to make are either calling the `test_read` or
`test_write` functions from above.
We could also invoke ioctls or other actions.

The [complete test harness](https://github.com/alastairreid/linux/blob/01f4b9331441a79e9ca38d7ddfddddd11d42dabf/samples/rust/rust_semaphore.rs#L278)
looks like this

```
/// Perform arbitrary sequence of file operations of length `n`
fn test_sequence_of_fileops<F: FileOperations>(file_state: &F, file: &File, n: usize) {
    for _ in 0..n {
        match AbstractValue::abstract_value() {
            0 => test_write(file_state, file, 128),
            1 => test_read(file_state, file, 128),
            _ => verifier::reject() // ignore this path
        }
    }
}

#[no_mangle]
pub fn test_fileops2() -> Result<()> {
    let registration = &RustSemaphore::init()?._dev;
    pr_info!("Initialized");

    let file_state: FileState = *mk_file_state::<Arc<Semaphore>, FileState>(registration)?;
    pr_info!("Got filestate");

    let file = File::make_fake_file();
    test_sequence_of_fileops(&file_state, &file, 4);

    Ok(())
}
```

And now we can compile, link and run the new test.


``` shell
make CC=wllvm -j16 samples/rust/rust_semaphore.o
llvm-link-11 samples/rust/rust_semaphore.bc samples/rust/.rust_semaphore.mod.o.bc rust/alloc.bc rust/kernel.bc rust/core.bc rust/rstubs.bc rust/.stubs.o.bc rust/verification_annotations.bc -o t.bc
klee --entry-point=test_fileops2 t.bc
```

This will generate a lot of output like this (and you will have to use ctrl-C
to stop KLEE).

```
KLEE: terminating blocked thread:0
```

This indicates that a thread has blocked waiting for input.
For the rust_semaphore device, threads will block if there have been
more reads than writes.
(We will return to the problem of what to do when a thread blocks later.)

This is not very interesting and we can prevent this from happening by
modifying the test to only read if there have been sufficient writes.
Obviously, doing this means that we will not explore some possible behaviours
so this should be done with care.

``` rust
let mut n = 0;
for _ in 0..steps {
    match AbstractValue::abstract_value() {
        0 => {
            test_write(file_state, file, 128);
            n = n + 1;
        },
        1 if n > 0 => {
            test_read(file_state, file, 128);
        },
        _ => verifier::reject() // ignore this path
    }
}
```

With this change, the output changes to look more like this.
(For readability, I am removing the KLEE WARNING lines.)

```
...
Initialized:0
Got filestate:0
Calling write:0
6%s: %pA:0
Called write:0
Calling read:0
Calling write:0
6%s: %pA:0
Called write:0
6%s: %pA:0
Called read:0
Calling write:0
Calling read:0
...
Rust semaphore sample (exit)
:0
Rust semaphore sample (exit)
:0
Rust semaphore sample (exit)
:0

KLEE: done: total instructions = 10237
KLEE: done: completed paths = 16
KLEE: done: generated tests = 8
```

The information about "Calling read/write" is not very informative,
but the final three lines tells us that

- In the course of exploring sequences of reads and writes of length 4, we explored 16 paths.
  (This makes sense since there are 16 sequences of length 4.)

- Eight paths were not rejected.
  (This makes sense since we reject some of those sequences to avoid blocking
  threads.)

This test has exhaustively tested all paths of length 4 (that do not deadlock)
and, because the device driver is quite simple, it completed quite quickly
(under a second).
Increasing the path length to 8 still completes in a second or so.
Further increasing the path length to 16 takes about 100 seconds (mostly due to all the
output printed by the test).

As we keep increasing the maximum path length, it becomes
impossible to exhaustively test every path in an acceptable time.
We can either hit ctrl-C when we get bored or give KLEE a timeout value
like 5 minutes or an hour and KLEE will randomly check some subset of the
paths.
Since KLEE is not testing every path, we cannot guarantee that no path
(up to the maximum path length) can fail but we do know that each of the paths
that KLEE explores cannot fail.

KLEE cannot give any guarantee about paths because (to a first approximation)
KLEE does a depth-first exploration of the paths.  Other tools such as bounded
model checkers perform a breadth-first exploration: exploring all shorter paths
before exploring longer paths.
Using a bounded model checker would give a more usable guarantee: that
all paths up to some length cannot fail.
It would be interesting to repeat this entire exercise with a bounded model
checker to see far we get.


# A simple verification harness: Arbitrary size reads and writes

In this section, we're going to take the original test and make
a different change.
(This will run into a limitation of KLEE so, for simplicity, we
are only going to consider a single write followed by a single read
and not an arbitrary sequence of operations.)

Let's take the original test at the start of this article
and make it just a little more flexible: instead of
using fixed data sizes of 128, let's use arbitrary sizes.
This will explore vaguely sensible values like 0, 3, 32 and very
large values like 123456789 and 0xffff_ffff_ffff_ffff.
In particular, KLEE will explore corner cases based on the conditions found in
the code it executes. If there is a check for zero, it will explore that;
if there is a maximum allowed read, it will explore that;
if it allocates memory, it will explore memory exhaustion.

Using the same `abstract_value` function as before, we can generalize the fixed
test above as follows.
(See [the final code](https://github.com/alastairreid/linux/blob/01f4b9331441a79e9ca38d7ddfddddd11d42dabf/samples/rust/rust_semaphore.rs#L248).)


``` rust
use verification_annotations;

#[no_mangle]
pub fn test_fileops2() -> Result<()> {
    let registration = &RustSemaphore::init()?._dev;
    pr_info!("Initialized");
    let file_state = *mk_file_state::<Arc<Semaphore>, FileState>(registration)?;
    pr_info!("Got filestate");
    let file = File::make_fake_file();

    let wlen = AbstractValue::abstract_value();
    let rlen = AbstractValue::abstract_value();

    test_write(&file_state, &file, wlen);
    test_read(&file_state, &file, rlen);

    Ok(())
}
```

Using KLEE to check this with the following commands.
(Hit ctrl-C after the KLEE command once you see the repeated pattern of output.)


``` shell
make CC=wllvm -j16 samples/rust/rust_semaphore.o
llvm-link-11 samples/rust/rust_semaphore.bc samples/rust/.rust_semaphore.mod.o.bc rust/alloc.bc rust/kernel.bc rust/core.bc rust/rstubs.bc rust/.stubs.o.bc rust/verification_annotations.bc -o t.bc
klee --entry-point=test_fileops2 t.bc
```

We get a lot of output including lines like the following:

```
KLEE: terminating blocked thread:0
```

Once again, we are running a lot of tests where a thread is blocked waiting for input.
In this case it is caused by writing zero bytes to the semaphore because
reading from an empty semaphore causes the reader to block.
(We will return to the problem of what to do when a thread blocks later.)

This is not very interesting and we can prevent this from happening by ensuring that the
initial write is non-zero.
This is done by adding the following constraint on the allowed values of wlen.
It can be added anywhere after the wlen value is created but KLEE is more
efficient if it occurs soon after wlen is created.
(Again, adding this restriction to workaround a limitation in our tools
is not ideal. We risk missing any bugs associated with the behaviours
that we are not testing so this is something to do with care and
either revisit later or to check using some other means such as testing or
fuzzing.)

``` rust
    verifier::assume(wlen != 0); // avoid read blocking on empty semaphore
```

We recompile, link and rerun KLEE in the usual way

``` shell
make CC=wllvm -j16 samples/rust/rust_semaphore.o
llvm-link-11 samples/rust/rust_semaphore.bc samples/rust/.rust_semaphore.mod.o.bc rust/alloc.bc rust/kernel.bc rust/core.bc rust/rstubs.bc rust/.stubs.o.bc rust/verification_annotations.bc -o t.bc
klee --entry-point=test_fileops2 t.bc
```

And KLEE produces around 30-40 lines of output including the following

```
KLEE: NOTE: found huge malloc, returning 0
KLEE: ERROR: (location information missing) concretized symbolic size
KLEE: NOTE: now ignoring this error at this location
RUST PANIC:0
...
KLEE: NOTE: found huge malloc, returning 0
RUST PANIC:0
```

These indicate two issues.

1) Picking very large values for wlen and rlen can cause memory allocation
failures.
2) KLEE does not support symbolic object sizes.


### Excessively large memory allocations

The first issue is that KLEE has found that it is possible for wlen and rlen to be
enormous which would cause the buffer allocation requests
in `make_reader` and `make_writer` to fail.
The fact that large buffer allocations can cause memory
allocation to fail is not overly surprising but the
fact that it causes the kernel to panic is an issue that
might need to be fixed.

We can confirm that this is the issue by temporarily adding
a constraint that will make the issue happen all the time.
(Also, I often add temporary restrictions like this just to check that the
verification tool is detecting such errors.)

``` rust
verifier::assume(rlen >= 0x8000_0000); // restrict to out of memory executions
```

So let's assume that we tried this or we stared at the code and
we think it is a bug[^is-it-a-bug] and we have filed a bug report on
that issue, how can we get KLEE to stop telling us about the bug so that we can
press on and find other bugs that we don't yet know about?

[^is-it-a-bug]:
    I am not sure that this is a bug or, if it is a bug, whether we should view
    it as a bug in Rust for Linux or in the test harness.
    I'm going to pretend that it is a bug because it lets me
    show a useful trick and because, in practice, some bug reports turn
    out not to be bugs.

We can prevent this from happening by constraining the
maximum values of wlen and rlen

``` rust
verifier::assume(wlen < 0x10000); // avoid out of memory
verifier::assume(rlen < 0x10000); // avoid out of memory
```

But, if we just add these constraints, then there is a risk that
we forget that they are just a temporary workaround
until the bug is fixed.
So, we prefer to write them in a stylized way.
Let's first assume that these alleged bugs have been assigned
the tracking numbers 2000 and 2001 respectively.
We will introduce global constants that say that the bugs
have not yet been fixed.

``` rust
static BUG_2000_FIXED: bool = false;
static BUG_2001_FIXED: bool = false;
```

We can then use those constants to turn the assumptions on and off
as follows.
This is exactly equivalent to the original constraints
but, once we think we have fixed the bug, we can quickly
flip the constant and test whether the problem goes away.
Once we are sure it is fixed, we will probably delete
the constant and the assumption but, until then,
this coding convention makes it easy to understand
that this assumption is just a temporary workaround
and why it was added.

``` rust
verifier::assume(BUG_2000_FIXED || wlen < 0x10000); // avoid out of memory until bug fixed
verifier::assume(BUG_2001_FIXED || rlen < 0x10000); // avoid out of memory until bug fixed
```

We can add these assumptions and rerun.

``` shell
make CC=wllvm -j16 samples/rust/rust_semaphore.o
llvm-link-11 samples/rust/rust_semaphore.bc samples/rust/.rust_semaphore.mod.o.bc rust/alloc.bc rust/kernel.bc rust/core.bc rust/rstubs.bc rust/.stubs.o.bc rust/verification_annotations.bc -o t.bc
klee --entry-point=test_fileops2 t.bc
```

Rerunning will now highlight the second issue.


### KLEE does not support symbolic object sizes

If you rerun, you will see that KLEE is not just reporting a WARNING, it is reporting an ERROR.

```
KLEE: ERROR: (location information missing) concretized symbolic size
```

KLEE's internal representation of memory requires that every
object has a known (concrete) size.
This is a limitation of KLEE (that is shared by several other tools) that is
going to prevent us from fully exploring all possible behaviours.
When KLEE finds that the size of an allocation request is symbolic,
it picks some concrete size and continues with that.
KLEE reports this as an ERROR because it indicates that KLEE is not
exploring all possible behaviours. In particular, suppose that KLEE
picked the size "42" and it did not find any problems, that would
definitely not mean that there are no problems.

The options at this point are not ideal.

1. We could switch to another tool (but KLEE is the only tool that we know works).

2. We could allocate a buffer of concrete length `N` but then pick a symbolic
   number `n` less than `N` and use a slice of the buffer.
   This would probably work in this case because we can easily change
   the buffer allocation code to over-allocate; it usually becomes hard in larger projects.

3. An alternative is that we could explore a small number of values.
   This risks missing bugs but it is not as bad as just using one value.

Let's look at the third option: exploring a small number of possible values.

We can do this by adding the following line.
(For efficiency reasons, this is best added just before the allocation (i.e.,
before the call to `test_write`.)

``` rust
let wlen = verifier::sample(5, wlen); // enumerate 5 possible values
```

The function `verifier::sample(n, e)` causes KLEE to explore `n different
values of the expression `e`.
It is not "sound"[^unsound-sample] because we are no longer trying all
possible values and we might miss bugs.

[^unsound-sample]:
    Since `verifier::sample` is not sound, it should
    possibly be renamed `verifier::unsound_sample`?

### The final version of this verification harness

With all those changes, our final verification harness looks as follows.

``` rust
#[no_mangle]
pub fn test_fileops2() -> Result<()> {
    let registration = &RustSemaphore::init()?._dev;
    pr_info!("Initialized");

    // get a FileState
    let file_state = *mk_file_state::<Arc<Semaphore>, FileState>(registration)?;
    pr_info!("Got filestate");

    let file = File::make_fake_file();

    let wlen = AbstractValue::abstract_value();
    let rlen = AbstractValue::abstract_value();
    verifier::assume(wlen != 0); // writes of length 0 don't bump semaphore
    verifier::assume(BUG_2000_FIXED || wlen < 0x10000);
    verifier::assume(BUG_2001_FIXED || rlen < 0x10000);

    // let rlen = verifier::sample(5, rlen); // enumerate 5 possible values
    // let wlen = verifier::sample(5, wlen); // enumerate 5 possible values

    // write some data *before* reading
    test_write(&file_state, &file, wlen);

    // read some data (will block if we have not written first)
    test_read(&file_state, &file, rlen);

    Ok(())
}
```

With that all done, we can make the amount of data we read be symbolic and watch the code fail

```
Rust semaphore sample (init)
:0
KLEE: WARNING ONCE: Alignment of memory from call "malloc" is not modelled. Using alignment of 8.
Initialized:0
Got filestate:0
KLEE: WARNING: klee_make_symbolic: renamed empty name to "unnamed"
Calling write:0
KLEE: NOTE: found huge malloc, returning 0
KLEE: ERROR: (location information missing) concretized symbolic size
KLEE: NOTE: now ignoring this error at this location
RUST PANIC:0
KLEE: ERROR: (location information missing) abort failure
KLEE: NOTE: now ignoring this error at this location
6%s: %pA:0
Called write:0
KLEE: WARNING: klee_make_symbolic: renamed empty name to "unnamed"
Calling read:0
KLEE: NOTE: found huge malloc, returning 0
RUST PANIC:0
6%s: %pA:0
Called read:0
Rust semaphore sample (exit)
:0
6%s: %pA:0
Called read:0
Rust semaphore sample (exit)
:0

KLEE: done: total instructions = 8340
KLEE: done: completed paths = 6
KLEE: done: generated tests = 4
```

## Summary

This was another monster post with lots of details that
described

- How to use the low-level [verification-annotations][Using
  verification-annotations] library to turn test harnesses
  into verification harnesses.

- Build a verification harness to verify sequences of device driver
  operations up to some length.

- Build a verification harness to verify device driver operations
  using arbitrary length buffers.

  This hit a limitation of KLEE: KLEE supports symbolic values
  but it does not support buffers whose length is symbolic.

- How to handle bugs that you have previously discovered
  but have not yet got a fix for.
  There is no point in detecting the bug again until you think the
  bug has been fixed but you don't want to  disable detection
  and then forget to re-enable it again later.

However, what we have done is a long way from ideal: this series of posts
is only the beginning of the process of using verification tools with
Rust for Linux and a lot more work is needed
before I would recommend that developers try using these tools
as part of their regular workflow.
Some of the limitations are


- The most serious limitation is that we did not use our [PropVerify library][Using PropVerify]
  with the result that the verification harnesses developed above
  can only be used with verification tools.
  In particular, a key idea in our project is that you should develop
  verification harnesses that can also serve as fuzzing harnesses
  (or fuzzing harnesses that can also serve as verification harnesses)
  (see [our paper][HATRA 2020]).
  It is therefore really annoying that we could not do that here.

  The main reason that we did not use PropVerify here was that Rust for Linux
  is not using Cargo and it seemed too hard to try to overcome that.
  I think that this would be easier in the current Rust for Linux version
  so this may be easy to fix.

  A much minor reason is that PropVerify currently depends on libstd --- it
  should not be too hard to modify it to support no-std environments.


- The verification harness only checks that the code does not
  break safety properties.
  It would be a lot better to find specific properties that we expect
  particular device drivers to satisfy and check for those.

- To avoid distractions, I have made little attempt at integrating
  my changes into the Rust for Linux codebase.
  That is, my changes need cleaned up and refactored by somebody
  who understands the build system and is thinking about
  the overall test/verification story and code structure.

- The Rust compiler recently changed from using LLVM-11 to using LLVM-12.
  It is hard for research tools to track the high rate of change in the
  Rust compiler and so we were not able to use KLEE on the latest version
  of Rust for Linux: we had to use a 4-month old version instead.

  This is unfortunate because Rust for Linux is changing very rapidly.

  If you want to try to update the ideas in this post to the latest Rust for
  Linux, [this branch](https://github.com/alastairreid/linux/tree/r4l_new_llvm12)
  is known to compile and might be a better starting point
  (assuming that you have a version of KLEE that can handle LLVM-12 bitcode
  files).

- We hit some limitations of KLEE

  - (Like many verification tools) KLEE does not currently support concurrency.
    This is a problem because concurrency is fundamental to what OS kernels do
    and also because one of the most important Rust mottos is "Fearless
    Concurrency!"

  - KLEE does not support symbolic allocation sizes.
    There are some possible partial workarounds but they are not perfect.

  - KLEE is primarily intended as a tool to find bugs and to generate high coverage
    testsuites and it is very good at that.
    We are using KLEE a little beyond its intended use when we try to
    use it to prove properties about code.
    It would probably be better to use a tool that performs a breadth-first
    exploration of code paths because that gives a more usable guarantee
    about the code.

I hope that this series of posts will inspire some of you to
try to fix some of the above issues.

Enjoy!

-----------


[WLLVM]:                          https://github.com/travitch/whole-program-llvm
[Rust For Linux]:                 https://github.com/Rust-for-Linux/linux
[RFC]:                            https://lore.kernel.org/lkml/20210414184604.23473-1-ojeda@kernel.org/
[RVT git repo]:                   {{site.gitrepo}}/
[KLEE]:                           https://klee.github.io/
[Using PropVerify]:               {{site.baseurl}}{% post_url 2020-09-03-using-propverify %}
[HATRA 2020]:                     https://alastairreid.github.io/papers/HATRA_20/
[Vectorized code 2]:              {{site.baseurl}}{% post_url 2021-05-15-verifying-vectorized-code2 %}
[Using KLEE with CoreUtils]:      {{site.baseurl}}{% post_url 2021-07-14-coreutils %}
[Using KLEE with Rust for Linux (part 1)]: {{site.baseurl}}{% post_url 2021-08-22-rust-on-linux-1 %}
[Using KLEE with Rust for Linux (part 2)]: {{site.baseurl}}{% post_url 2021-08-23-rust-on-linux-2 %}
[Using KLEE with Rust for Linux (part 3)]: {{site.baseurl}}{% post_url 2021-08-24-rust-on-linux-3 %}
[Using PropVerify]:               {{site.baseurl}}{% post_url 2020-09-03-using-propverify %}
[PropTest]:                       https://github.com/AltSysrq/proptest/
[Using verification-annotations]: {{site.baseurl}}{% post_url 2020-09-02-using-annotations %}
