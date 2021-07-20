---
layout: post
title: Using KLEE on Rust-for-Linux (part 2)
---

![Linux logo](https://cdn.kernel.org/pub/linux/kernel/v2.0/Logo.gif){: style="float: left; width: 10%; padding: 1%"}
Using tools like  the [KLEE] symbolic execution tool or bounded model checkers
with Linux kernel code is still a bit of a black art.
The [first part of this series][Using KLEE with Rust for Linux (part 1)]
on using [KLEE] on the [Rust for Linux]
considered what we would want to check.
This second part, digs deeply into how to prepare the codebase for use with
[LLVM]-based tools like [KLEE].
(Warning: it may contain *far more detail* than most people are interested in.)
The [final part][Using KLEE with Rust for Linux (part 3)] will
show how we can build simple verification frameworks.
The code changes described in this post are
in [this branch of my fork of Rust for Linux](https://github.com/alastairreid/linux/tree/klee).

The basic steps involved in this are

1. Use WLLVM to build the rLinux[^rLinux] code (i.e., C code) while
   emitting LLVM bitcode files.

    [^rLinux]:
        Recall that I made up the shortened name "rLinux" to refer
        to the "rest of Linux": those parts of Linux that are not written
        in Rust.

2. Modify the Makefile to get `rustc` to emit LLVM bitcode files.

3. Write 'stub functions' that represent the rLinux functions called
   by the Rust for Linux infrastructure.

4. Link the right bitcode files together.

5. Test whether it works by using [KLEE] to concretely
   execute the code.

Note that, in this part, we will ignore the idea of verifying something: we will
just focus on building something we can give to KLEE.
In [the third part][Using KLEE with Rust for Linux (part 3)]
we will then use [KLEE] to symbolically execute the code.

A quick heads up before we get started: there are a lot
of steps here, a few of them are a bit slow.
Hopefully though, you can just cut-and-paste sequences
of commands into your shell in the background
while you do something more interesting.
You may also want to extend the Makefile in the Linux kernel to
make it easier to repeat some of the steps.

Also, a reminder from the first part that the following does not work with
the latest version of Rust for Linux.
The Rust compiler moves very fast and this causes problems for verification tools.
In particular, what I describe is based on KLEE that recently added support
for LLVM-11 but the latest Rust compiler and Rust for Linux relies on LLVM-12.
All the detailed instructions are therefore for [an older version of Rust for
Linux](https://github.com/Rust-for-Linux/linux/commit/e71e2ab9686e168aebb086d8bd2643b79f20106e)
from late May instead of the current version.[^partial-fix]

[^partial-fix]:
    If you want to update this to work with the current version of 
    Rust for Linux, I have
    [another branch](https://github.com/alastairreid/linux/tree/r4l_new_llvm12)
    that compiles but, since I was not able to use KLEE, it is untested
    and it has not been cleaned up to make it suitable for merging into the
    main branch.
    But it may be useful as a guide to what needs to be changed.


## Installing tools

The first step is installing some tools needed to build Rust for Linux
and the WLLVM tool.

``` shell
rustup component add rust-src
rustup override set nightly-2021-02-20
rustup component add rustfmt clippy
cargo install --locked --version 0.56.0 bindgen

sudo apt-get -y install \
    git fakeroot build-essential ncurses-dev xz-utils \
    libssl-dev bc flex libelf-dev bison \
    clang-11 file

pip3 install --upgrade wllvm
```

And also building the [RVT][RVT git repo] docker image
that we will need later.[^LLVM11-support]

[^LLVM11-support]:
    Note that the docker image has partial LLVM11 support.
    It is sufficient for the way that we will
    use KLEE with Linux but we do not recommend it for more
    general use because `#[test]` support is currently broken.

``` shell
# install docker
sudo apt-get install -y docker
sudo groupadd docker
sudo usermod -aG docker $USER
# you may have to log out and back in to make the last command take effect

# install RVT and build a docker image
git clone https://github.com/project-oak/rust-verification-tools.git rvt
cd rvt
env LLVM11=yes docker/build

# set an environment variable to point to the Rust verification tools directory
export RVT_DIR=`pwd`
```


## Using WLLVM to build the rLinux code

The first step is generating [LLVM] bitcode files
using [WLLVM].
This will be very similar to the way Linux is normally
built except that we need to build LLVM bitcode files
instead of x86 ELF files.


We are going to build bitcode for rLinux using WLLVM in some approximation
of the instructions in these posts:
<https://arkivm.github.io/2016/12/26/linux-whole-program-bc/> and <https://blog.xiexun.tech/linux-bc-custom-opt.html>

There is one important wrinkle (i.e., hack).
To build the Linux kernel, we need to use clang-11.
But, to build using WLLVM, we need to use a program called "clang"
and there is no direct way to say "use clang-11 instead".
Fortunately, there is a standard hack for this situation:
we create a symlink from clang to clang-11 and ensure
that the symlink is on our path.
We could put the symlink in ~/local/bin but then we are bound
to forget that it is there and, three months from now, it will break
something in a very confusing way. Instead, it is better
to put it in some temporary directory and add it to our PATH
in the interactive command line.[^you-will-forget]

[^you-will-forget]:
    Warning: if you log out and restart in the middle of trying these commands,
    you will almost certainly forget to rerun the `export PATH` and
    `export LLVM_COMPILER` commands.

``` shell
mkdir foobin
ln -s /usr/bin/clang-11 foobin/clang
export PATH=$PWD/foobin:$PATH
export LLVM_COMPILER=clang
```

Now, we clone Rust for Linux (including the entire Linux kernel),
tweak the Makefile and build it.

Note that we are cloning a fork that includes all the changes
described below instead of the Rust for Linux repo.
If you prefer, you can fork the original repo and make the changes yourself.


``` shell
# git clone https://github.com/Rust-for-Linux/linux.git
git clone --branch klee https://github.com/alastairreid/linux.git
cd linux
```

To make `rustc` generate LLVM bitcode files for Rust code, we
make the following change to `Makefile`.

```
+++ b/Makefile
@@ -806,6 +806,10 @@ else ifdef CONFIG_RUST_OPT_LEVEL_Z
 KBUILD_RUSTCFLAGS += -Copt-level=z
 endif

+KBUILD_RUSTCFLAGS += --emit=llvm-bc
+
 # Tell gcc to never replace conditional load with a non-conditional one
 KBUILD_CFLAGS  += $(call cc-option,--param=allow-store-data-races=0)
```

Now we can configure the Linux kernel by turning on all of the Rust for Linux
options.

``` shell
make menuconfig
# Enable General setup:Rust support (last entry in General setup menu)
# Enable Kernel hacking:Sample kernel code:Rust samples and enable all the example code
# Enable Device drivers:Android:Android drivers:* (scroll down a long way to find Android!
# Enable all drivers (not sure what we want so just build all of them)
```

Finally, we can build the kernel using WLLVM[^verbosity]

[^verbosity]:
    A useful tip if you want extra verbosity is to add "V=1" to the
    make command.

``` shell
make KCFLAG='-O0' CC=wllvm -j16 -k
make KCFLAG='-O0' CC=wllvm -j16 -k modules
```

These commands will fail with a lot of linker warnings about duplicate symbols because
the git repo will contain 'stubs' that are used to replace bits of the rLinux
code.
For our current purposes, this is ok but, of course, we should
really fix the Makefile to only link the stubs when compiling
for verification.
(We use the '-k' flag to force make to keep going after the linking failure.)


Using WLLVM to build the kernel results in the normal ELF .o files
plus some LLVM bitcode files that we need for use with KLEE.
WLLVM creates three files for every .c file: .o, .bc and .cmd.
The .cmd file is the command or a makefile (or something like that).
For example, the files created for the sample device driver `rust_chrdev` are

```
$ ls -al samples/rust/rust_chrdev* samples/rust/.rust_chrdev*
-rw-r--r-- 1 adreid adreid   191 Aug 12 15:17 samples/rust/.rust_chrdev.ko.cmd
-rw-r--r-- 1 adreid adreid   112 Aug 12 12:19 samples/rust/.rust_chrdev.mod.cmd
-rw-r--r-- 1 adreid adreid  6616 Aug 12 15:17 samples/rust/.rust_chrdev.mod.o.bc
-rw-r--r-- 1 adreid adreid 30082 Aug 12 15:17 samples/rust/.rust_chrdev.mod.o.cmd
-rw-r--r-- 1 adreid adreid  3305 Aug 12 12:18 samples/rust/.rust_chrdev.o.cmd
-rw-r--r-- 1 adreid adreid 36020 Aug 12 12:18 samples/rust/rust_chrdev.bc
-rw-r--r-- 1 adreid adreid 10160 Aug 12 15:17 samples/rust/rust_chrdev.ko
-rw-r--r-- 1 adreid adreid    28 Aug 12 12:19 samples/rust/rust_chrdev.mod
-rw-r--r-- 1 adreid adreid   561 Aug 12 15:17 samples/rust/rust_chrdev.mod.c
-rw-r--r-- 1 adreid adreid  2904 Aug 12 15:17 samples/rust/rust_chrdev.mod.o
-rw-r--r-- 1 adreid adreid  7888 Aug 12 12:18 samples/rust/rust_chrdev.o
-rw-r----- 1 adreid adreid  2193 Aug  9 11:23 samples/rust/rust_chrdev.rs
```

The `.mod.c` file is a machine generated file created for every kernel module.
The `rust_chrdev.bc` file is the bitcode corresponding to `rust_chrdev.ko`.

*[If you are wondering why we built the entire rLinux kernel as
bitcode even though we decided above that we were not going to try to
verify the rLinux code, the reason is that we need a few of the bitcode files
and it is easier to build the whole thing than to try to just build the parts
that we need.
(There is lots of room to optimize this process!)]*


## Using KLEE with the Linux bitcode files

[KLEE] is a symbolic execution tool that runs LLVM bitcode
files using symbolic inputs.
But, before we start writing verification harnesses that generate
symbolic inputs, we can test whether the bitcode files we
have generated kinda work with KLEE.
We need a version of [KLEE] that supports LLVM-11 so we will run all future
KLEE commands in the docker image that we built earlier.[^could-be-slimmer]

[^could-be-slimmer]:
    For my convenience, I am using the docker image that I use for all
    Rust verification work. This contains far more than you need because
    we will not be using all of the features of our work. This makes it
    slower to build and much larger than it has to be since the only part you
    will need is KLEE.

    If you want a smaller, leaner, faster process, it is not too hard to [build KLEE
    yourself](https://klee.github.io/getting-started/) or, if you like docker,
    to use the KLEE build system to build your own LLVM-11 docker image.
    But see the special KLEE runtime build commands later in this section.

From now on we will run all KLEE-related commands in an interactive docker session
in a separate terminal window

``` shell
$RVT_DIR/docker/run
```

### Undefined references

The following command (that is run in the Docker session)
attempts to concretely execute the kernel initialization
code of one of the Rust device drivers in KLEE.
Even this minimalist and not very interesting test will fail: our immediate
goal is not to get KLEE to run *something interesting* but
to get KLEE to run *anything* and we will use the error/warning messages
to figure out what we need to fix.

``` shell
klee --entry-point=init_module samples/rust/rust_chrdev.bc
```

This prints a bunch of warnings of which the most obvious are warnings about
undefined references.
These come in two forms:
references to Rust code (these have really long, mangled names);
and references to C code (these have short, comprehensible names).


```
KLEE: WARNING: undefined reference to function: _RNvMNtCsbDqzXfLQacH_6kernel4fileNtB2_4File14make_fake_file
KLEE: WARNING: undefined reference to function: _RNvMNtCsbDqzXfLQacH_6kernel4fileNtB2_4File8from_ptr
...
KLEE: WARNING: undefined reference to function: _RNvXs_NtCsbDqzXfLQacH_6kernel6chrdevNtB4_4CdevNtNtNtCsbDqzXfLQacH_4core3ops4drop4Drop4drop
KLEE: WARNING: undefined reference to function: __rust_alloc
KLEE: WARNING: undefined reference to function: __rust_dealloc
KLEE: WARNING: undefined reference to variable: __this_module
KLEE: WARNING: undefined reference to function: alloc_chrdev_region
KLEE: WARNING: undefined reference to function: unregister_chrdev_region
```

The problem is that, although our change to the Makefile compiles
all the Rust code to bitcode, it does not link the bitcode files together.
We will start by linking in the missing Rust code.
Through a bit of analysis and a bit of experiment, I found that the following
files were sufficient to resolve the references to Rust symbols.

``` shell
llvm-link-11 rust/alloc.bc rust/kernel.bc rust/core.bc samples/rust/.rust_chrdev.mod.o.bc samples/rust/rust_chrdev.bc -o t.bc
$RVT_DIR/docker/run klee --entry-point=init_module t.bc
```

### Rebuilding KLEE's runtime system

The linking step fixes the undefined references but now we have an error

```
error: linking module flags 'wchar_size': IDs have conflicting values in 'memcmp64_Debug+Asserts.bc' and 't.bc'
```

This refers to the file `memcpy64_Debug+Asserts.bc` that is an essential part
of KLEE's runtime system.
The problem here is that when we built KLEE, this file was built as if we were
compiling usermode programs but we're dealing with the Linux kernel here and it
is built with different compiler flags.
Grepping the Makefile finds the flag we need to use: `-fshort-wchar`.

```
$ grep wchar Makefile
    fno-strict-aliasing -fno-common -fshort-wchar -fno-PIE \
```

The fix is to add this flag to  the KLEE runtime build system and rebuild.
We will do this in the interactive Docker session but beware that if you
quit and restart the session, then the changes will be lost – so don't quit and
restart.

``` shell
$ pushd ~/klee

# Apply the following change to runtime/CMakeLists.txt

diff --git a/runtime/CMakeLists.txt b/runtime/CMakeLists.txt
index 6ee6f830..ea97366e 100644
--- a/runtime/CMakeLists.txt
+++ b/runtime/CMakeLists.txt
@@ -50,6 +50,8 @@ foreach (bc_architecture ${bc_architectures})
                     "Optimization (\"${bc_optimization}\") for runtime library unknown.")
         endif ()

+        list(APPEND local_flags -target x86_64-unknown-unknown-elf -fshort-wchar)
+
         # Define suffix-specific optimizations
         set("LIB_BC_FLAGS_${bc_architecture}_${bc_optimization}" ${local_flags})
     endforeach ()

$ cd build
$ make -j16
$ sudo make install
$ popd
```

With that change, KLEE is able to concretely run the kernel module
initializer although it produces a lot of warnings about undefined C functions

```
KLEE: WARNING: undefined reference to function: __init_waitqueue_head
KLEE: WARNING: undefined reference to function: __platform_driver_register
KLEE: WARNING: undefined reference to function: __wake_up
KLEE: WARNING: undefined reference to function: add_device_randomness
KLEE: WARNING: undefined reference to function: alloc_chrdev_region
...
KLEE: WARNING ONCE: calling external: alloc_chrdev_region(94909129121336, 0, 2, 94909128291856) at [no debug info]
KLEE: ERROR: (location information missing) failed external call: alloc_chrdev_region
...
KLEE: done: total instructions = 1191
KLEE: done: completed paths = 1
KLEE: done: generated tests = 1
```

We are making progress but the error (and all the warnings) indicates that it
is time to create a mock Linux layer.


## Create a mock Linux layer

All of the C functions that are called from Rust code are accessed via the file
`rust/bindings_generated.rs`: a file generated by `binder`.
For each of the C functions called by Rust, we need to create an
implementation.
These implementations do not need to be complete (after all, we could just use
the real rLinux implementation) but they need to be sufficient to let us run
tests and detect bugs.
Since we are at an early stage of testing, this bar is quite low: the code only
has to be type-correct and memory-safe.
In most cases, this can be a function with either an empty body or it will
return 0 or a value like `core::ptr::null_mut()`.
Of course, these will need to be improved as we get more serious but,
for now, we just want something to run without spurious errors.

For example here is the stub for `cdev_add`.

``` rust
#[no_mangle]
unsafe extern "C" fn cdev_add(_arg1: *mut cdev, _arg2: dev_t, _arg3: c_types::c_uint) -> c_types::c_int {
    0
}
```

Some of the other mock functions that we create are the following.
The code for these can be found at [rust/rstubs.rs in the klee branch](https://github.com/alastairreid/linux/blob/klee/rust/rstubs.rs).

``` rust
unsafe extern "C" fn cdev_init(_arg1: *mut cdev, _arg2: *const file_operations);
unsafe extern "C" fn cdev_add(_arg1: *mut cdev, _arg2: dev_t, _arg3: c_types::c_uint) -> c_types::c_int;
unsafe extern "C" fn cdev_del(_arg1: *mut cdev);
unsafe extern "C" fn __init_waitqueue_head(...);
unsafe extern "C" fn __wake_up(...);
unsafe extern "C" fn prepare_to_wait_exclusive(...);
unsafe extern "C" fn schedule();
unsafe extern "C" fn finish_wait(_wq_head: *mut wait_queue_head, _wq_entry: *mut wait_queue_entry);
unsafe extern "C" fn __mutex_init(_lock: *mut mutex, _name: *const c_types::c_char, _key: *mut lock_class_key);
unsafe extern "C" fn mutex_lock(_lock: *mut mutex);
unsafe extern "C" fn mutex_unlock(_lock: *mut mutex);
unsafe extern "C" fn add_device_randomness(_arg1: *const c_types::c_void, _arg2: c_types::c_uint);
unsafe extern "C" fn rng_is_initialized() -> bool_;
unsafe extern "C" fn wait_for_random_bytes() -> c_types::c_int;
unsafe extern "C" fn get_random_bytes(_buf: *mut c_types::c_void, _nbytes: c_types::c_int);
unsafe extern "C" fn alloc_chrdev_region(...);
unsafe extern "C" fn register_chrdev_region(...);
unsafe extern "C" fn unregister_chrdev_region(_arg1: dev_t, _arg2: c_types::c_uint);
unsafe extern "C" fn kernel_param_lock(_mod_: *mut module);
unsafe extern "C" fn kernel_param_unlock(_mod_: *mut module);
unsafe extern "C" fn slab_is_available() -> bool_;
unsafe extern "C" fn vm_insert_page(...);
unsafe extern "C" fn __free_pages(_page: *mut page, _order: c_types::c_uint);
unsafe extern "C" fn register_sysctl(...);
unsafe extern "C" fn unregister_sysctl_table(_table: *mut ctl_table_header);
unsafe extern "C" fn misc_register(_misc: *mut miscdevice) -> c_types::c_int;
unsafe extern "C" fn misc_deregister(_misc: *mut miscdevice);
```

Although most of our mock functions are trivial, there are a few functions
that it is important to give a real implementation for.
For example, we need the memory allocator to actually allocate memory.
Normally this calls `krealloc` in rLinux but we need it to call
KLEE's memory allocator function `realloc`.
So we replace this code (in
[rust/kernel/allocator.rs](https://github.com/alastairreid/linux/blob/klee/rust/kernel/allocator.rs#L18)).

``` rust
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // `krealloc()` is used instead of `kmalloc()` because the latter is
        // an inline function and cannot be bound to as a result.
        unsafe { bindings::krealloc(ptr::null(), layout.size(), bindings::GFP_KERNEL) as *mut u8 }
    }
```

with this code

``` rust
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        extern "C" {
            pub fn realloc(arg1: *const c_types::c_void, arg2: usize) -> *mut c_types::c_void;
        }
        realloc(ptr::null(), layout.size()) as *mut u8
    }
```

We also have a [non-trivial mocking layer for
`printk`](https://github.com/alastairreid/linux/blob/klee/rust/stubs.c#L7)
although that has to be
written in C (because the function type has varargs).[^printk-approx]

[^printk-approx]:
    There are multiple ways that printk can be mocked.
    One way is to have it format the string and print that to stdout.
    For simplicity, we chose to have it print only the format string
    and to print by calling the KLEE function `klee_print_expr` instead.
    There's no right or wrong answer here -- although my abuse
    of `klee_print_expr` may be closer to wrong :-).


After writing the mock layer, we can relink and run it in KLEE

``` shell
make CC=wllvm -j16
llvm-link-11 samples/rust/rust_chrdev.bc samples/rust/.rust_chrdev.mod.o.bc rust/alloc.bc rust/kernel.bc rust/core.bc rust/rstubs.bc rust/.stubs.o.bc -o t.bc
klee --entry-point=init_module t.bc
klee --entry-point=cleanup_module t.bc
```

## Create minimal test harness for FileOperations

As a very first attempt at creating a test, we will write a simple,
conventional test for the `rust_chrdev` device driver and use KLEE
to execute the bitcode.
(At this stage, we will not do any symbolic execution: that
is left to the [next part of this series][Using KLEE with Rust for Linux (part 3)].)

One key abstraction used in Rust for Linux is the `UserSlicePtrWriter`
that is used for writing some data into a memory buffer.
The following function allocates a buffer and uses that to build
a `UserSlicePtrWriter`.

``` rust
fn make_writer(len: usize) -> UserSlicePtrWriter {
    let mut data: Vec<u8> = Vec::with_capacity(len);
    unsafe { UserSlicePtr::new(data.as_mut_ptr() as *mut c_types::c_void, len).writer() }
}
```

With this function, we can create a simple test function that exercises
the read system call.
Note that this test does not check that we see the expected behavior, it
only checks that the code is memory safe (using KLEE's built in checks for
this).
Exactly what properties a device driver is supposed to satisfy vary
from one device to the next and this is just a generic initial test.


``` rust
fn test_read<F: FileOperations>(file_state: &F, file: &File, len: usize) {
    pr_info!("Calling read");
    let mut data = make_writer(len);
    let offset: u64 = 0;
    match FileOperations::read(file_state, file, &mut data, offset) {
        Err(Error(rc)) => pr_info!("read error: {}", rc),
        Ok(sz) => pr_info!("read {} bytes", sz),
    }
    pr_info!("Called read");
}
```

We also need one important change to the Rust for Linux infrastructure.
For testing purposes, we need to be able to create a File object.
These normally contain a pointer to an rLinux struct `file`  but, since
we are not linking rLinux into the codebase we don't need to populate all
the fields and we don't have the rLinux code that would populate all the
fields.
For now, we are going to take the bold step of replacing the pointer to
the file struct with... a null pointer.
To do this, we extend
[rust/kernel/file.rs](https://github.com/alastairreid/linux/blob/klee/rust/kernel/file.rs#L46)
with the function `make_fake_file`.

``` rust
/// For testing purposes, we can make a file out of nothing
/// Note that this only works as long as the code being tested
/// does not use any of the other methods.
/// (This is a hack)
pub fn make_fake_file() -> File {
    let fptr: *const bindings::file = core::ptr::null();
    unsafe { File::from_ptr(fptr) }
}
```

With the help of these  functions, we can write a simple test function

``` rust
#[no_mangle]
pub fn test_fileops() -> Result<()> {
    let ctx = ();
    let f: Box<RustFile> = RustFile::open(&ctx)?;

    let file = File::make_fake_file();
    test_read(&*f, &file, 128);

    Ok(())
}
```

Which can be run using

``` shell
make CC=wllvm -j16 samples/rust/rust_chrdev.o
llvm-link-11 samples/rust/rust_chrdev.bc samples/rust/.rust_chrdev.mod.o.bc rust/alloc.bc rust/kernel.bc rust/core.bc rust/rstubs.bc rust/.stubs.o.bc -o t.bc
klee --entry-point=test_fileops t.bc
```

producing this output

```
KLEE: WARNING: undefined reference to ...
...
Calling read:0
KLEE: WARNING ONCE: Alignment of memory from call "malloc" is not modelled. Using alignment of 8.
6%s: %pA:0
Called read:0

KLEE: done: total instructions = 58
KLEE: done: completed paths = 1
KLEE: done: generated tests = 1
```

Which shows the output from running the test code.
(The string "6%s: %pA:0" is a format string passed to printk because the mock
printk implementation is not very good.)


## Testing rust_semaphore

The above test is not very strong.
In theory, it could fail on a null pointer dereference but, that is not very
likely.
In part, this is because it is just our first attempt.
but it is also because the `rust_chrdev` driver doesn't do very much
so there is not a lot to test.

We can fix this by switching to `rust_semaphore` which supports which
supports read, write and ioctl system calls.

If these were actual files, we might want to check that writing data and then
reading it back returned the original data (say).
Unfortunately, even an obvious property like that need not hold
because devices are not required to behave exactly like files.
In particular, the semaphore device ignores the data value: it just
increments the semaphore by the amount of data being written.

In principle, every device can behave differently from 
any other so a single function will not be sufficient to test them.
But some devices share some common properties so it should be possible to
create stronger checkers that can be used across multiple device drivers.
For now though, we must content ourselves with checking that the code does
not fail KLEE's standard safety checks.

``` rust
fn make_reader(len: usize) -> UserSlicePtrReader {
    let mut data: Vec<u8> = Vec::with_capacity(len);
    unsafe { UserSlicePtr::new(data.as_mut_ptr() as *mut c_types::c_void, len).reader() }
}

fn test_read<F: FileOperations>(file_state: &F, file: &File, len: usize) {
    pr_info!("Calling read");
    let mut data = make_writer(len);
    let offset: u64 = 0;
    match FileOperations::read(file_state, file, &mut data, offset) {
        Err(Error(rc)) => pr_info!("read error: {}", rc),
        Ok(sz) => pr_info!("read {} bytes", sz),
    }
    pr_info!("Called read");
}
```

``` rust
#[no_mangle]
pub fn test_fileops() -> Result<()> {
    // 1) Use RustSemaphore::init() to create module state sema
    // 2) Use FileState::open(sema) to get Box<FileState>
    // 3) Test the following operations
    //    - read // should block unless semaphore >= 1
    //    - write // increments semaphore by either 1 or write size (can't figure out which)
    //    - todo: ioctl too

    let registration = &RustSemaphore::init()?._dev;
    pr_info!("Initialized");

    // get a FileState
    let file_state = *mk_file_state::<Arc<Semaphore>, FileState>(registration)?;
    pr_info!("Got filestate");

    let file = File::make_fake_file();

    // write some data *before* reading
    test_write(&file_state, &file, 128);

    // read some data (will block if we have not written first)
    test_read(&file_state, &file, 128);

    Ok(())
}

fn mk_file_state<T: Sync, FS: FileOpener<T> + FileOperations>(reg: &Registration<T>) -> Result<FS::Wrapper> {
    let sema: &T = &reg.context;
    FS::open(&sema)
}
```

Which can be compiled, linked and run by

``` shell
make CC=wllvm -j16 samples/rust/rust_semaphore.o
llvm-link-11 samples/rust/rust_semaphore.bc samples/rust/.rust_chrdev.mod.o.bc rust/alloc.bc rust/kernel.bc rust/core.bc rust/.stubs.o.bc rust/rstubs.bc -o t.bc
klee --entry-point=test_fileops t.bc
```

Which produces this output

```
KLEE: output directory is "/usr/local/google/home/adreid/rust/rvt/downloads/linux/klee-out-46"
KLEE: Using STP solver backend
KLEE: WARNING ONCE: unsupported intrinsic llvm.sideeffect
KLEE: WARNING: undefined reference to ...
...
KLEE: WARNING ONCE: String not terminated by \0 passed to one of the klee_ functions
Rust semaphore sample (init)
:0
KLEE: WARNING ONCE: Alignment of memory from call "malloc" is not modelled. Using alignment of 8.
Initialized:0
Got filestate:0
Calling write:0
6%s: %pA:0
Called write:0
Calling read:0
6%s: %pA:0
Called read:0
Rust semaphore sample (exit)
:0

KLEE: done: total instructions = 8110
KLEE: done: completed paths = 1
KLEE: done: generated tests = 1
```

This example is slightly more interesting and there are the beginnings
of some reusable testing functions that can be used for multiple drivers.


## Summary

Whew! If you made it this far, then you now know how to

- Generate LLVM bitcode files for the Linux kernel by compiling it with WLLVM.
- Generate LLVM bitcode files for the Rust for Linux code by adding
  a flag to `KBUILD_RUSTCFLAGS` in the Makefile.
- Write a mock-Linux library to use instead of "rLinux" functions.
- Link the bitcode files for a given Rust kernel module
- *Concretely* execute the code in [KLEE].
- Write simple test harnesses to exercise the Rust for Linux device
  drivers[^cleanup-needed]

[^cleanup-needed]:
    To let me focus on the issues around KLEE and testing, I have completely
    ignored any software engineering issues.
    In particular, I made random changes in files that need refactored,
    changes in a docker container that will be lost the moment that you
    close the container and other minor abominations.
    To make all this usable in practice, this all needs to be cleaned up
    but this post was already far, far too long

But, the testing itself is still fairly limited and, in its current state, is
not likely to catch bugs.
The point of this whole series is not to
show that it is *useful* to use KLEE with Rust code but to figure
out whether it is even *possible*.
These are just the beginning steps on what could be a very long journey.

In the [final part of this series][Using KLEE with Rust for Linux (part 3)] we
will look at how to write verification harnesses and use [KLEE]
to *symbolically* execute them.


-----------


[WLLVM]:                          https://github.com/travitch/whole-program-llvm
[Rust For Linux]:                 https://github.com/Rust-for-Linux/linux
[RFC]:                            https://lore.kernel.org/lkml/20210414184604.23473-1-ojeda@kernel.org/
[RVT git repo]:                   {{site.gitrepo}}/
[KLEE]:                           https://klee.github.io/
[LLVM]:                           https://llvm.org/
[Using PropVerify]:               {{site.baseurl}}{% post_url 2020-09-03-using-propverify %}
[HATRA 2020]:                     https://alastairreid.github.io/papers/HATRA_20/
[Vectorized code 2]:              {{site.baseurl}}{% post_url 2021-05-15-verifying-vectorized-code2 %}
[Using KLEE with CoreUtils]:      {{site.baseurl}}{% post_url 2021-07-14-coreutils %}
[Using KLEE with Rust for Linux (part 1)]: {{site.baseurl}}{% post_url 2021-08-22-rust-on-linux-1 %}
[Using KLEE with Rust for Linux (part 2)]: {{site.baseurl}}{% post_url 2021-08-23-rust-on-linux-2 %}
[Using KLEE with Rust for Linux (part 3)]: {{site.baseurl}}{% post_url 2021-08-24-rust-on-linux-3 %}
