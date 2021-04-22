# Copyright 2021 The Rust verification tools Authors.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# this rule comes first to make sure that it is the default target
default: simd_emulation.bc

RUSTC ?= rustc
RSFLAGS = --emit=llvm-bc --crate-type=lib -O -Ctarget-feature=+sse2
RSFLAGS += --edition=2018

LINK = llvm-link-${LLVM_VERSION}

RS_SRCS := $(wildcard src/*.rs)
RS_OBJS := $(patsubst src/%.rs, build/%.bc, $(RS_SRCS))

OBJS := $(RS_OBJS)

build/%.bc: src/%.rs
	mkdir -p $(dir $@)
	$(RUSTC) $(RSFLAGS) $^ -o $@

simd_emulation.bc: $(OBJS)
	$(LINK) $(OBJS) -o $@

clean::
	$(RM) -r build
	$(RM) simd_emulation.bc

# End of Makefile
