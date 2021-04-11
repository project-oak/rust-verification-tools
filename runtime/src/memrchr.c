// Copyright 2021 The Rust verification tools Authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#include <string.h>

void *memrchr(const void *s, int c, size_t n) {
    void *r = NULL;
    for(size_t i = 0; i < n && *(char*)s != '\0'; ++i, ++s) {
        if (*(char*)s == c) {
            r = (void*)s;
        }
    }
    return r;
}
