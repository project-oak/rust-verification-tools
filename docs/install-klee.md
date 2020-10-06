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
sudo apt-get install  cmake bison flex libboost-all-dev python perl
sudo apt-get install  build-essential curl libcap-dev git cmake libncurses5-dev
sudo apt-get install  python3-minimal python3-pip unzip libtcmalloc-minimal4 libgoogle-perftools-dev
sudo apt-get install  libsqlite3-dev doxygen clang

pip3 install tabulate wllvm lit toml colored
```

or using `brew` and `pip3` on OSX

```
# Optional: consider running 'brew upgrade' first?
brew install curl git cmake python unzip gperftools sqlite3 doxygen bash
brew install cryptominisat
pip3 install tabulate wllvm lit toml colored
```

Add this to `~/.bash_profile` so that any installed libraries can be found

```
export PATH=/usr/include/bin:$PATH
export LD_LIBRARY_PATH=/usr/include/lib:$LD_LIBRARY_PATH
export LDFLAGS="-L/usr/include/lib $LDFLAGS"
export DYLD_LIBRARY_PATH="/usr/include/lib:$DYLD_LIBRARY_PATH"
```

Also, add this to `~/.bash_profile` on OSX

```
export PATH=$HOME/bin:$PATH
export LD_LIBRARY_PATH=$HOME/lib:$LD_LIBRARY_PATH
export LDFLAGS="-L$HOME/lib $LDFLAGS"
export DYLD_LIBRARY_PATH="$HOME/lib:$DYLD_LIBRARY_PATH"

export PATH=$HOME/homebrew/bin:$PATH
export LD_LIBRARY_PATH=$HOME/homebrew/lib:$LD_LIBRARY_PATH
export LDFLAGS="-L$HOME/homebrew/lib $LDFLAGS"
export DYLD_LIBRARY_PATH="$HOME/homebrew/lib:$DYLD_LIBRARY_PATH"
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
git submodule init && git submodule update
mkdir build
cd build
cmake ..
make
sudo make install
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
make -j
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
git merge nowack/handle_global_variables reid/lazy_intrinsic_rejection

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

[Project Oak]:                     https://github.com/project-oak/oak/blob/main/README.md
[KLEE]:                            https://klee.github.io
