// Copyright 2020 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/////////////////////////////////////////////////////////////////
// FFI wrapper for Crux-mir static simulator tool
/////////////////////////////////////////////////////////////////

#[cfg(crux)]
extern crate crucible;

// pub mod crucible::symbolic;
// pub use self::symbolic::Symbolic;

// Create an abstract value of type <T>
//
// This should only be used on types that occupy contiguous memory
// and where all possible bit-patterns are legal.
// e.g., u8/i8, ... u128/i128, f32/f64
pub fn abstract_value<T: crucible::Symbolic>() -> T {
    T::symbolic(concat!("symb_", line!(), "_", column!()))
}

// Add an assumption
pub fn assume(cond: bool) {
    crucible::crucible_assume!(cond)
}

// Reject the current execution with a verification failure.
//
// In almost all circumstances, report_error should
// be used instead because it generates an error message.
pub fn abort() {
    crucible::crucible_assert!(false)
}

// Reject the current execution path with a verification success.
// This is equivalent to assume(false)
// and the opposite of report_error.
//
// Typical usage is in generating symbolic values when the value
// does not meet some criteria.
pub fn reject() -> ! {
    crucible::crucible_assume!(false);
    panic!("should have been rejected!");
}

// Detect whether the program is being run symbolically in KLEE
// or being replayed using the kleeRuntest runtime.
//
// This is used to decide whether to display the values of
// variables that may be either symbolic or concrete.
pub fn is_replay() -> bool {
    false
}

// Reject the current execution with a verification failure
// and an error message.
pub fn report_error(message: &str) {
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
    println!("VERIFIER_EXPECT: {}", msg)
}

// Declare that failure is the expected behaviour
pub fn expect(msg: Option<&str>) {
    match msg {
        None => println!("VERIFIER_EXPECT: should_panic"),
        Some(msg) => println!("VERIFIER_EXPECT: should_panic(expected = \"{}\")", msg)
    }
}

/////////////////////////////////////////////////////////////////
// End
/////////////////////////////////////////////////////////////////
