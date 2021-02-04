// Copyright 2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/////////////////////////////////////////////////////////////////
// FFI wrapper for SeaHorn symbolic execution tool
/////////////////////////////////////////////////////////////////

use std::default::Default;
use core::panic::PanicInfo;

pub use crate::traits::*;

extern {
    fn __VERIFIER_error() -> !;
    fn __VERIFIER_assume(pred: i32);
}

#[no_mangle]
fn spanic(_info: &PanicInfo) -> ! {
    abort();
}

/// Reject the current execution with a verification failure.
///
/// In almost all circumstances, `report_error` should
/// be used instead because it generates an error message.
pub fn abort() -> ! {
    unsafe { __VERIFIER_error(); }
}

/// Assume that condition `cond` is true
///
/// Any paths found must satisfy this assumption.
pub fn assume(pred: bool) {
    if ! pred {
        unsafe { __VERIFIER_assume(0); }
    }
}

/// Reject the current execution path with a verification success.
/// This is equivalent to `assume(false)`
/// and the opposite of `report_error(...)`.
///
/// Typical usage is in generating symbolic values when the value
/// does not meet some criteria.
pub fn reject() -> ! {
    assume(false);
    panic!("Unreachable, should have been rejected!");
}

/// Detect whether the program is being run symbolically in KLEE
/// or being replayed using the kleeRuntest runtime.
///
/// This is used to decide whether to display the values of
/// variables that may be either symbolic or concrete.
pub fn is_replay() -> bool {
    panic!("SeaHorn doesn't support replay.")
}

/// Reject the current execution with a verification failure
/// and an error message.
pub fn report_error(message: &str) -> ! {
    // Mimic the format of klee_report_error
    // (We don't use klee_report_error because it is not
    // supported by the kleeRuntest library.)
    eprintln!("SEAHORN: ERROR:{}", message);
    abort();
}

/// Declare that failure is the expected behaviour
pub fn expect_raw(msg: &str) {
    eprintln!("VERIFIER_EXPECT: {}", msg)
}

/// Declare that failure is the expected behaviour
pub fn expect(msg: Option<&str>) {
    match msg {
        None => eprintln!("VERIFIER_EXPECT: should_panic"),
        Some(msg) => eprintln!("VERIFIER_EXPECT: should_panic(expected = \"{}\")", msg)
    }
}

macro_rules! make_nondet {
    ($typ:ty, $ext:ident, $v:expr) => {
        extern { fn $ext() -> $typ; }
        impl VerifierNonDet for $typ {
            fn verifier_nondet(self) -> Self {
                unsafe { $ext() }
            }
        }
    };
}

make_nondet!(u8, __VERIFIER_nondet_u8, 0);
make_nondet!(u16, __VERIFIER_nondet_u16, 0);
make_nondet!(u32, __VERIFIER_nondet_u32, 0);
make_nondet!(u64, __VERIFIER_nondet_u64, 0);
// make_nondet!(u128, __VERIFIER_nondet_u128, 0);
make_nondet!(usize, __VERIFIER_nondet_usize, 0);

make_nondet!(i8, __VERIFIER_nondet_i8, 0);
make_nondet!(i16, __VERIFIER_nondet_i16, 0);
make_nondet!(i32, __VERIFIER_nondet_i32, 0);
make_nondet!(i64, __VERIFIER_nondet_i64, 0);
// make_nondet!(i128, __VERIFIER_nondet_i128, 0);
make_nondet!(isize, __VERIFIER_nondet_isize, 0);

make_nondet!(f32, __VERIFIER_nondet_f32, 0.0);
make_nondet!(f64, __VERIFIER_nondet_f63, 0.0);

impl VerifierNonDet for bool {
    fn verifier_nondet(self) -> Self {
        let c = u8::verifier_nondet(0u8);
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
