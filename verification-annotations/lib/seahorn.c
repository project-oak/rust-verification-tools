// Copyright 2020-2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

void __VERIFIER_error() {
  fprintf(stderr, "ERROR: a verification assertion failed.");
  exit(1);
}

void __VERIFIER_assume(int pred) {
  if (pred == 0) {
    fprintf(stderr, "ERROR: a verification assumption has been violated.");
    exit(1);
  }
}

uint8_t   __VERIFIER_nondet_u8()    { return 0; }
uint16_t  __VERIFIER_nondet_u16()   { return 0; }
uint32_t  __VERIFIER_nondet_u32()   { return 0; }
uint64_t  __VERIFIER_nondet_u64()   { return 0; }
uintptr_t __VERIFIER_nondet_usize() { return 0; }

int8_t    __VERIFIER_nondet_i8()    { return 0; }
int16_t   __VERIFIER_nondet_i16()   { return 0; }
int32_t   __VERIFIER_nondet_i32()   { return 0; }
int64_t   __VERIFIER_nondet_i64()   { return 0; }
intptr_t  __VERIFIER_nondet_isize() { return 0; }

float     __VERIFIER_nondet_f32()   { return 0; }
double    __VERIFIER_nondet_f64()   { return 0; }
