# Installing the Rust compiler

One of the great things about Rust is that it only has one compiler
and a single standard library.
This means that verification tools can all use the Rust compiler
as the frontend for parsing, typechecking, handling modules, etc.

The [Rust compiler works](https://blog.rust-lang.org/2016/04/19/MIR.html)
in four stages:
first, it compiles Rust source code to HIR (the high-level IR);
HIR is converted to MIR (the mid-level IR);
MIR is converted to LLVM IR (the low-level virtual machine IR);
and, finally, it compiles LLVM IR down to machine code.

There seem to two main classes of verification tool for Rust:
those that use MIR code and those that use LLVM IR code.
Several verification tools for C already use LLVM IR code
so I decided to focus my initial efforts on using LLVM-based tools
with Rust.

Whichever type of tool you are using, you are probably going to
have to do two things:

1. Link any tools you are using against the version of LLVM being
   used in the Rust compiler.

2. Ensure that you have either MIR or LLVM IR for the standard
   Rust libraries (libcore, libstd, etc.)

So, the first step is building the Rust compiler and libraries
from source.
This is not too hard but it takes about an hour on my laptop
so you should plan on starting the build process before lunch/dinner and hoping that it is
finished by the time you are done.

I'm going to do all this work in a directory called "rust" so let's start by
creating that directory, installing a few tools and downloading the Rust source code.


todo: the following dependencies are for KLEE not rustc
need to find the real dependencies of rustc

```
mkdir $HOME/rust && cd $HOME/rust

# On Linux, do this
sudo apt-get install build-essential curl libcap-dev git cmake \
  libncurses5-dev python-minimal python-pip unzip libtcmalloc-minimal4 \
  libgoogle-perftools-dev libsqlite3-dev doxygen

# On OSX, do this (not tested as thoroughly)
brew install curl git cmake python unzip gperftools sqlite3 doxygen bash

pip3 install tabulate wllvm lit toml colored termcolor

# Download the Rust source code
git clone https://github.com/rust-lang/rust.git
```


By default, The Rust compiler and libraries support all targets.
But you probably only need one (probably X86) so the first step is
configuring the build configuration so that you don't spend (a lot of)
time building versions that you don't need.

```
cd rust
cp config.toml.example config.toml

# It is a good idea to edit config.toml to change this line 
#     #targets = "AArch64;ARM;Hexagon;MSP430;Mips;NVPTX;PowerPC;RISCV;Sparc;SystemZ;WebAssembly;X86"
# to this line 
#     targets = "X86"
# it will make the build process significantly shorter
```

Now the slow bit: building Rust.

Since I am interested in LLVM, I want to make sure that the library includes
LLVM IR "bitcode" for the libraries.
So I am going to pass the `-Cembed-bitcode=yes` flag to make sure this
happens.
[It is possible that the Rust standard libraries include LLVM IR bitcode
by default?
This all seemed to be in flux in April/May 2020 as I was figuring
out how to do things so I wanted to be doubly-sure that the
libraries would include the information that I needed.
So, it is possible that you don't need the `RUSTFLAGS_STAGE_NOT_0`
flag. But it shouldn't do any harm.]

Verification tools [have different optimization needs from normal
compilation][Overify].
When compiling, you want to generate code that takes full advantage
of any special features of your hardware: the cache, special instructions,
loop unrolling, etc.
But, when verifying, you just want your code to be simple and uncluttered.
I haven't really explored what this means yet but one thing that I might want
to try is disabling some of the architecture-specific optimizations such as
vectorization and the use of architecture specific features like Intel's SSE,
AVX, etc.

```
RUSTFLAGS_STAGE_NOT_0="-Cembed-bitcode=yes"
# todo: disable vectorization, sse, avx, etc.
./x.py build
```

That will take a while to build but, once it is done, let's make it the default
compiler.

```
rustup toolchain link stage1 build/x86*/stage1
rustup default stage1
```

We're now done building Rust so let's add the LLVM compiler inside Rust to our
path and pop up into the parent directory.

```
export PATH=$PATH:`echo $HOME/rust/rust/build/x86_*/llvm/bin`

cd ..
```



-----------------------



[Rust language]:                   https://www.rust-lang.org
[Rust book]:                       https://doc.rust-lang.org/book/
[Cargo tool]:                      https://doc.rust-lang.org/cargo/
[Rustonomicon]:                    https://doc.rust-lang.org/nomicon/
[Rust fuzzing]:                    https://github.com/rust-fuzz

[Project Oak]:                     https://github.com/project-oak/oak/blob/main/README.md
[KLEE]:                            https://klee.github.io
