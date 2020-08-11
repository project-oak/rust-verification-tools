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

pip3 install tabulate wllvm lit

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
rustup toolchain link stage2 build/x86*/stage2
rustup default stage2
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
[Sealed Rust]:                     https://ferrous-systems.com/blog/sealed-rust-the-pitch/
[Rust verification working group]: https://rust-lang-nursery.github.io/wg-verification/
[Rust verification workshop]:      https://sites.google.com/view/rustverify2020

[Project Oak]:                     https://github.com/project-oak/oak/blob/main/README.md
[Haskell]:                         https://haskell.org/
[nofib benchmark suite]:           https://link.springer.com/chapter/10.1007/978-1-4471-3215-8_17

[CBMC]:                            https://github.com/diffblue/cbmc/pull/4894
[Crust]:                           https://github.com/uwplse/crust
[Crux-mir]:                        https://github.com/GaloisInc/mir-verifier
[Electrolysis]:                    https://github.com/Kha/electrolysis
[Haybale]:                         https://github.com/PLSysSec/haybale
[Cargo-KLEE]:                      https://gitlab.henriktjader.com/pln/cargo-klee
[KLEE Rust]:                       https://github.com/jawline/klee-rust
[KLEE]:                            https://klee.github.io
[LibHoare]:                        https://github.com/nrc/libhoare
[MIRAI]:                           https://github.com/facebookexperimental/MIRAI
[Miri]:                            https://github.com/rust-lang/miri
[PRUSTI]:                          https://github.com/viperproject/prusti-dev
[RustBelt]:                        https://plv.mpi-sws.org/rustbelt/
[RustFuzz]:                        https://github.com/rust-fuzz
[RustHorn]:                        https://github.com/hopv/rust-horn
[SeaHorn]:                         https://seahorn.github.io
[Seer]:                            https://github.com/dwrensha/seer
[SMACK]:                           https://github.com/smackers/smack

[contracts crate]:                 https://gitlab.com/karroffel/contracts
[Viper rust-contracts]:            https://github.com/viperproject/rust-contracts
[arbitrary crate]:                 https://github.com/rust-fuzz/arbitrary

[librarification]:                 http://smallcultfollowing.com/babysteps/blog/2020/04/09/libraryification/
[verifier crate]:                  https://crates.io/crates/verifier
[verifier benchmarks]:             https://github.com/soarlab/rust-benchmarks

[astrauskas:oopsla:2019]:          https://alastairreid.github.io/RelatedWork/papers/astrauskas:oopsla:2019/
[baranowski:atva:2018]:            https://alastairreid.github.io/RelatedWork/papers/baranowski:atva:2018/
[jung:popl:2017]:                  https://alastairreid.github.io/RelatedWork/papers/jung:popl:2017/
[jung:popl:2020]:                  https://alastairreid.github.io/RelatedWork/papers/jung:popl:2020/
[lindner:indin:2018]:              https://alastairreid.github.io/RelatedWork/papers/lindner:indin:2018/
[lindner:indin:2019]:              https://alastairreid.github.io/RelatedWork/papers/lindner:indin:2019/
[matsushita:esop:2020]:            https://alastairreid.github.io/RelatedWork/papers/matsushita:esop:2020/
[toman:ase:2015]:                  https://alastairreid.github.io/RelatedWork/papers/toman:ase:2015/
[Overify]:                         https://alastairreid.github.io/RelatedWork/papers/wagner:hotos:2013

[Rust verification papers]:        https://alastairreid.github.io/RelatedWork/notes/rust-language/
[Lean]:                            https://alastairreid.github.io/RelatedWork/notes/lean-theorem-prover/
[SV-COMP]:                         https://alastairreid.github.io/RelatedWork/notes/sv-competition/

[verification competitions]:       https://alastairreid.github.io/verification-competitions/
[Rust verification tools]:         https://alastairreid.github.io/rust-verification-tools/
[joining Oak post]:                https://alastairreid.github.io/joining-google/
