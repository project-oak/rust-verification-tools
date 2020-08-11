# Installation


### Installing KLEE's dependencies

We are going to build KLEE from source for two reasons:

1. KLEE is built on top of LLVM and we want to make sure that
   it is built on top of the same version of LLVM that Rust
   is built with so we build from source.

2. In the process of doing this work, I had to fix/extend KLEE
   to deal with several minor issues so we need to use a
   development version of KLEE.
   Specifically, we need to use a version from late August 2020
   and we need to merge in some draft pull requests.

The KLEE installation process is described
[on these pages](https://klee.github.io/build-llvm9/).
In practice though, I find it slightly easier to
consult the [Dockerfile](https://hub.docker.com/r/klee/klee/dockerfile)
and the [Dockerfiles of all of KLEE's dependencies](https://hub.docker.com/u/klee).

For convenience, I am going to give instructions but, if my instructions don't
work for you, you should consult one of those sources.

We are going to install
minisat (OSX only),
STP, some (optional) tests and KLEE itself.

For simplicity, I will install everything in the default location (`/usr/local`
on Linux, `$HOME/homebrew` on OSX).
But, if you prefer to install somewhere else such as `$HOME/local`
you can add the flag `-DCMAKE_INSTALL_PREFIX=$HOME/local`  when
invoking `cmake` and you should define this environment variable
to avoid linking errors involving `libkleeRuntest`

```
RUSTFLAGS="-C link-args=-Wl,--library-path=$HOME/local/lib" 
```

### Installing the Rust compiler and libraries

Before installing KLEE, you will also need to build the Rust compiler
and libraries following [these instructions](../docs/installation.md).
This is quite easy but very slow so I recommend that you start
that process first.


### Installing standard packages

Start by installing some standard packages
using `apt-get` and `pip3` on Linux

```
# Optional: consider running 'sudo apt-get update; sudo apt-get upgrade' first?
sudo apt-get install  cmake bison flex libboost-all-dev python perl minisat
sudo apt-get install  build-essential curl libcap-dev git cmake libncurses5-dev
sudo apt-get install  python-minimal python-pip unzip libtcmalloc-minimal4 libgoogle-perftools-dev
sudo apt-get install  libsqlite3-dev doxygen

pip3 install tabulate wllvm lit
```

or using `brew` and `pip3` on OSX

```
# Optional: consider running 'brew upgrade' first?
brew install curl git cmake python unzip gperftools sqlite3 doxygen bash
brew install cryptominisat
pip3 install tabulate wllvm lit
```

### Our work directory

For convenience, I'm going to assume that you want to build
all the tools in `$HOME/rust`.
If you prefer to put it elsewhere, it should be easy enough to
change the instructions below.

```
mkdir ~/rust
cd ~/rust
```

### Building minisat (OSX only)

On Linux, we can install minisat with `apt-get install minisat`.
On OSX, we need to build minisat from source.

```
# from https://github.com/stp/stp
git clone https://github.com/stp/minisat.git
cd minisat
mkdir build
cd build
cmake ..
make
make install
cd ../..

```

### Building STP

```
# from https://klee.github.io/build-stp/
git clone https://github.com/stp/stp.git
cd stp
git checkout tags/2.3.3
mkdir build
cd build
cmake ..
# sudo is required because some files are installed in /usr/lib/python2.7
sudo make install
cd ../..
```

### Googletest

```
curl -OL https://github.com/google/googletest/archive/release-1.7.0.zip
unzip release-1.7.0.zip
```


### Building KLEE

Building KLEE is pretty much the same as
[on these pages](https://klee.github.io/build-llvm9/)
except that we are going to make KLEE use the
LLVM compiler used by Rust.
This LLVM/Rust compiler source is in `rust/src/llvm-project/llvm`
and it will have been built int `rust/build/x86_*/llvm/bin`.

Before building KLEE, it is essential to build the Rust compiler
and libraries following [these instructions](../docs/installation.md).

A minimal install of KLEE looks something like this.

```
git clone https://github.com/klee/klee.git
cd klee

# Merge in pending pull requests: welcome to the bleeding edge!
git remote add reid   https://github.com/alastairreid/klee.git
git remote add nowack https://github.com/MartinNowack/klee.git
git fetch --all
git merge nowack/handle_global_variables reid/lazy_intrinsic_rejection reid/cxa_thread_atexit_impl

mkdir build
cd build
cmake \
  -DENABLE_SOLVER_STP=ON \
  -DLLVMCC=`which clang` \
  -DENABLE_KLEE_UCLIBC=OFF \
  -DLLVM_CONFIG_BINARY=`ls $HOME/rust/rust/build/x86_*/llvm/bin/llvm-config` \
  -DENABLE_UNIT_TESTS=ON \
  -DGTEST_SRC_DIR=$HOME/rust/googletest-release-1.7.0 \
  ..
```

Once configured, build it, run the testsuite and install.

```
make -j
make check
sudo make install
```

You should now be able to run `klee --help` and see a myriad of
verification options.




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
