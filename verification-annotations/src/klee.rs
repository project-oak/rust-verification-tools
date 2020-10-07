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

pub use crate::traits::*;

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

/// Create instance for any type consisting of contiguous memory
/// where all bit-patterns are legal values of the type.
macro_rules! make_verifier_nondet {
    ($typ:ident) => {
        impl VerifierNonDet for $typ {
            fn verifier_nondet(self) -> Self {
                let mut r = self;
                unsafe {
                    let data = std::mem::transmute(&mut r);
                    let length = std::mem::size_of::<$typ>();
                    let null = 0 as *const i8;
                    klee_make_symbolic(data, length, null)
                }
                return r;
            }
        }
    };
}

make_verifier_nondet!(u8);
make_verifier_nondet!(u16);
make_verifier_nondet!(u32);
make_verifier_nondet!(u64);
make_verifier_nondet!(u128);
make_verifier_nondet!(usize);

make_verifier_nondet!(i8);
make_verifier_nondet!(i16);
make_verifier_nondet!(i32);
make_verifier_nondet!(i64);
make_verifier_nondet!(i128);
make_verifier_nondet!(isize);

make_verifier_nondet!(f32);
make_verifier_nondet!(f64);

impl VerifierNonDet for bool {
    fn verifier_nondet(self) -> Self {
        let c = VerifierNonDet::verifier_nondet(0u8);
        assume(c == 0 || c == 1);
        c == 1
    }
}

impl <T: VerifierNonDet + Default> AbstractValue for T {
    fn abstract_value() -> Self {
        Self::verifier_nondet(Self::default())
    }
}

impl <T: VerifierNonDet + Default> Symbolic for T {
    fn symbolic(_desc: &'static str) -> Self {
        Self::verifier_nondet(Self::default())
    }
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


#[macro_export]
macro_rules! assert {
    ($cond:expr,) => { $crate::assert!($cond) };
    ($cond:expr) => { $crate::assert!($cond, "assertion failed: {}", stringify!($cond)) };
    ($cond:expr, $($arg:tt)+) => {{
        if ! $cond {
            let message = format!($($arg)+);
            eprintln!("VERIFIER: panicked at '{}', {}:{}:{}",
                      message,
                      std::file!(), std::line!(), std::column!());
            $crate::abort();
        }
    }}
}

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {{
        let left = $left;
        let right = $right;
        $crate::assert!(
            left == right,
            "assertion failed: `(left == right)` \
             \n  left: `{:?}`,\n right: `{:?}`",
            left,
            right)
    }};
    ($left:expr, $right:expr, $fmt:tt $($arg:tt)*) => {{
        let left = $left;
        let right = $right;
        $crate::assert!(
            left == right,
            concat!(
                "assertion failed: `(left == right)` \
                 \n  left: `{:?}`, \n right: `{:?}`: ", $fmt),
            left, right $($arg)*);
    }};
}

#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr) => {{
        let left = $left;
        let right = $right;
        $crate::assert!(
            left != right,
            "assertion failed: `(left != right)` \
             \n  left: `{:?}`,\n right: `{:?}`",
            left,
            right)
    }};
    ($left:expr, $right:expr, $fmt:tt $($arg:tt)*) => {{
        let left = $left;
        let right = $right;
        $crate::assert!(
            left != right,
            concat!(
                "assertion failed: `(left != right)` \
                 \n  left: `{:?}`, \n right: `{:?}`: ", $fmt),
            left, right $($arg)*);
    }};
}

/////////////////////////////////////////////////////////////////
// End
/////////////////////////////////////////////////////////////////
