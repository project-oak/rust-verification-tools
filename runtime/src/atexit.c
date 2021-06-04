int __cxa_atexit(void (*fn)(void*),
                 void *arg,
                 void *dso_handle) {
  return 0;
}

// This variant is part of more recent glibc versions and
// is required by the Rust standard library
int __cxa_thread_atexit_impl(void (*fn)(void*), void *arg, void *dso_handle) {
  return __cxa_atexit(fn, arg, dso_handle);
}
