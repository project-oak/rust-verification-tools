// Copyright 2021 The Rust verification tools Authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Very basic pthread support
//
// - only supports one thread!
// - all functions do nothing and report success
// - functions that return values return 0 or NULL

#include "pthread.h"

/***/

int pthread_attr_init (pthread_attr_t *__attr) {
        return 0;
}

int pthread_attr_getstack (const pthread_attr_t * __attr,
                           void ** __stackaddr,
                           size_t * __stacksize) {
        *__stackaddr = 0;
        *__stacksize = 0;
        return 0;
}

int pthread_attr_destroy (pthread_attr_t *__attr) {
        return 0;
}

int pthread_getattr_np (pthread_t __th, pthread_attr_t *__attr) {
        return 0;
}

/***/

int pthread_cond_init (pthread_cond_t * __cond, const pthread_condattr_t * __cond_attr) {
        return 0;
}

int pthread_cond_destroy (pthread_cond_t *__cond) {
        return 0;
}

int pthread_cond_signal (pthread_cond_t *__cond) {
        return 0;
}

int pthread_cond_wait (pthread_cond_t * __cond, pthread_mutex_t * __mutex) {
        return 0;
}

/***/

int pthread_condattr_init (pthread_condattr_t *__attr) {
        return 0;
}

int pthread_condattr_destroy (pthread_condattr_t *__attr) {
        return 0;
}

int pthread_condattr_setclock (pthread_condattr_t *__attr, __clockid_t __clock_id) {
        return 0;
}

/***/

static void *specific_value;

void *pthread_getspecific (pthread_key_t __key) {
        return specific_value;
}

int pthread_setspecific (pthread_key_t __key, const void *__pointer) {
        specific_value = (void*)__pointer;
        return 0;
}

int pthread_key_create (pthread_key_t *__key, void (*__destr_function) (void *)) {
        return 0;
}

int pthread_key_delete (pthread_key_t __key) {
        return 0;
}

/***/

int pthread_mutex_init (pthread_mutex_t *__mutex, const pthread_mutexattr_t *__mutexattr) {
        return 0;
}

int pthread_mutex_destroy (pthread_mutex_t *__mutex) {
        return 0;
}

int pthread_mutex_lock (pthread_mutex_t *__mutex) {
        return 0;
}

int pthread_mutex_unlock (pthread_mutex_t *__mutex) {
        return 0;
}

/***/

int pthread_mutexattr_destroy (pthread_mutexattr_t *__attr) {
        return 0;
}
int pthread_mutexattr_init (pthread_mutexattr_t *__attr) {
        return 0;
}
int pthread_mutexattr_settype (pthread_mutexattr_t *__attr, int __kind) {
        return 0;
}

/***/

int pthread_rwlock_init (pthread_rwlock_t * __rwlock, const pthread_rwlockattr_t * __attr) {
        return 0;
}

int pthread_rwlock_destroy (pthread_rwlock_t *__rwlock) {
        return 0;
}

int pthread_rwlock_rdlock (pthread_rwlock_t *__rwlock) {
        return 0;
}

int pthread_rwlock_unlock (pthread_rwlock_t *__rwlock) {
        return 0;
}

int pthread_rwlock_wrlock (pthread_rwlock_t *__rwlock) {
        return 0;
}

/***/

pthread_t pthread_self (void) {
        return (pthread_t)0;
}

// End
