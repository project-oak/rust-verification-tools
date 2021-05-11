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

use crate::traits::*;

impl<T: AbstractValue> VerifierNonDet for T {
    fn verifier_nondet(self) -> Self {
        T::abstract_value()
    }
}

impl<T: crucible::Symbolic> AbstractValue for T {
    fn abstract_value() -> Self {
        // We assume the string argument is just for reporting to the user, and
        // doesn't affect the results.
        let r: T = crucible::Symbolic::symbolic("");
        r
    }
}

impl<T: crucible::Symbolic + Default> Symbolic for T {
    fn symbolic(desc: &'static str) -> Self {
        let r: T = crucible::Symbolic::symbolic(desc);
        r
    }
}

/// Assume that condition `cond` is true
///
/// Any paths found must satisfy this assumption.
pub fn assume(cond: bool) {
    crucible::crucible_assume!(cond)
}

/// Reject the current execution with a verification failure.
///
/// In almost all circumstances, `report_error` should
/// be used instead because it generates an error message.
pub fn abort() {
    crucible::crucible_assert!(false)
}

/// Reject the current execution path with a verification success.
/// This is equivalent to `assume(false)`
/// and the opposite of `report_error(...)`.
///
/// Typical usage is in generating symbolic values when the value
/// does not meet some criteria.
pub fn reject() -> ! {
    crucible::crucible_assume!(false);
    panic!("should have been rejected!");
}

/// Detect whether the program is being run symbolically in KLEE
/// or being replayed using the kleeRuntest runtime.
///
/// This is used to decide whether to display the values of
/// variables that may be either symbolic or concrete.
pub fn is_replay() -> bool {
    panic!("crux doesn't support replay")
}

/// Reject the current execution with a verification failure
/// and an error message.
pub fn report_error(message: &str) {
    crucible::crucible_assert!(false, "VERIFIER: ERROR: {}", message);
}

/// Declare that failure is the expected behaviour
pub fn expect_raw(_msg: &str) {
    panic!("not implemented")
}

/// Declare that failure is the expected behaviour
pub fn expect(_msg: Option<&str>) {
    panic!("not implemented")
}

#[macro_export]
macro_rules! assert {
    ($cond:expr) => {
        $crate::crucible::crucible_assert!(
            $cond,
            "VERIFIER: assertion failed: {}",
            stringify!($cond)
        );
    }; // ($cond:expr,) => { ... };
       // ($cond:expr, $($arg:tt)+) => { ... };
}

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        $crate::verifier::assert!(($left) == ($right));
    }; // ($left:expr, $right:expr,) => { ... };
       // ($left:expr, $right:expr, $($arg:tt)+) => { ... };
}

#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr) => {
        $crate::verifier::assert!(($left) != ($right));
    }; // ($left:expr, $right:expr,) => { ... };
       // ($left:expr, $right:expr, $($arg:tt)+) => { ... };
}

/////////////////////////////////////////////////////////////////
// End
/////////////////////////////////////////////////////////////////
