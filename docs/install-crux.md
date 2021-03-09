---
layout: page
title: Crux-Mir installation
---

The best way to install crux (aka mir-verifier) is to follow the instructions on
[crux's GitHub page](https://github.com/GaloisInc/mir-verifier).

For convenience, instructions for installing crux and its dependencies are
provided below.


We are going to install Haskell, Rust, mir-json, Yices and crux.

Where possible `apt` is used. 
Everything else is installed under `$HOME`.


### Installing Haskell

``` shell
sudo apt install cabal-install ghc
cabal new-update
cabal user-config update
```

Make sure `PATH` includes `$HOME/.cabal/bin`.


### Installing Rust

Install rust using rustup.
mir-json requires nightly-2020-03-22 so we will get that.

``` shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly-2020-03-22 --force
rustup default nightly-2020-03-22
rustup component add --toolchain nightly-2020-03-22 rustc-dev
```

Make sure `PATH` includes `$HOME/.cargo/bin`.

### Install mir-json

``` shell
git clone git@github.com:GaloisInc/mir-json.git
cd mir-json
RUSTC_WRAPPER=./rustc-rpath.sh cargo install --locked
```

### Install Yices

``` shell
git clone git@github.com:SRI-CSL/yices2.git
cd yices2
autoconf
./configure --prefix="$HOME/.local"
make
make install
```

Make sure `PATH` includes `$HOME/.local/bin`

### Building Crux

``` shell
git clone git@github.com:GaloisInc/mir-verifier.git
cd mir-verifier
git submodule update --init
cabal v2-build
./translate_libs.sh
cabal v2-install crux-mir
```

### Shell init script

Add the following lines to your shell init script (assuming crux was cloned into
`$HOME/mir-verifier`).

``` shell
export PATH=$HOME/.local/bin:$HOME/.cabal/bin:$HOME/.cargo/bin:$PATH
export CRUX_RUST_LIBRARY_PATH=$HOME/mir-verifier/rlibs
```
### Testing

Run crux's test suite:

``` shell
cd mir-verifier
cabal v2-test
# [...]
# All 254 tests passed (322.16s)
# Test suite test: PASS
# Test suite logged to:
# [...]
# 1 of 1 test suites (1 of 1 test cases) passed.
```
