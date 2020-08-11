// Copyright 2020 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/////////////////////////////////////////////////////////////////
// FFI wrapper for KLEE symbolic execution tool
/////////////////////////////////////////////////////////////////

use std::default::Default;
use std::os::raw;

#[link(name = "kleeRuntest")]
extern "C" {
    fn klee_make_symbolic(data: *mut raw::c_void, length: usize, name: *const raw::c_char);
    fn klee_assume(cond: usize);
    fn klee_abort() -> !;
    fn klee_silent_exit(_ignored: u32) -> !;
    fn klee_is_replay() -> i32;
}

// Create an abstract value of type <T>
//
// This should only be used on types that occupy contiguous memory
// and where all possible bit-patterns are legal.
// e.g., u8/i8, ... u128/i128, f32/f64
pub fn abstract_value<T: Default>() -> T {
    let mut r = T::default();
    unsafe {
        let data = std::mem::transmute(&mut r);
        let length = std::mem::size_of::<T>();
        let null = 0 as *const i8;
        klee_make_symbolic(data, length, null)
    }
    return r;
}

// Add an assumption
pub fn assume(cond: bool) {
    unsafe { klee_assume(if cond { 1 } else { 0 }) }
}

// Reject the current execution with a verification failure.
//
// In almost all circumstances, report_error should
// be used instead because it generates an error message.
pub fn abort() -> ! {
    unsafe { klee_abort() }
}

// Reject the current execution path with a verification success.
// This is equivalent to assume(false)
// and the opposite of report_error.
//
// Typical usage is in generating symbolic values when the value
// does not meet some criteria.
pub fn reject() -> ! {
    unsafe { klee_silent_exit(0) }
}

// Detect whether the program is being run symbolically in KLEE
// or being replayed using the kleeRuntest runtime.
//
// This is used to decide whether to display the values of
// variables that may be either symbolic or concrete.
pub fn is_replay() -> bool {
    unsafe { klee_is_replay() != 0 }
}

// Reject the current execution with a verification failure
// and an error message.
pub fn report_error(message: &str) -> ! {
    // Mimic the format of klee_report_error
    // (We don't use klee_report_error because it is not
    // supported by the kleeRuntest library.)
    eprintln!("KLEE: ERROR:{}", message);
    abort();
}

// Check an assertion
pub fn verify(cond: bool) {
    if !cond {
        report_error("verification failed");
    }
}

// Declare that failure is the expected behaviour
pub fn expect_raw(msg: &str) {
    eprintln!("VERIFIER_EXPECT: {}", msg)
}

// Declare that failure is the expected behaviour
pub fn expect(msg: Option<&str>) {
    match msg {
        None => eprintln!("VERIFIER_EXPECT: should_panic"),
        Some(msg) => eprintln!("VERIFIER_EXPECT: should_panic(expected = \"{}\")", msg)
    }
}

/////////////////////////////////////////////////////////////////
// End
/////////////////////////////////////////////////////////////////
