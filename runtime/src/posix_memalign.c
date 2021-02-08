// Copyright 2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#include <errno.h>
#include <malloc.h>
#include <stdlib.h>

int posix_memalign(void **memptr, size_t alignment, size_t size) {
        if (size == 0) { // allocate a unique address for size 0
                size = 1;
        }

        void *addr = memalign(alignment, size);
        if (!addr) {
                // *memptr is not modified on failure
                return ENOMEM;
        }

        *memptr = addr;
        return 0;
}
