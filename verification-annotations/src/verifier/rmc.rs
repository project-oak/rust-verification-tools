// Copyright 2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/////////////////////////////////////////////////////////////////
// FFI wrapper for RMC model checker
/////////////////////////////////////////////////////////////////

use std::convert::TryInto;

pub use crate::traits::*;

extern "C" {
    fn __VERIFIER_error() -> !;
    fn __VERIFIER_assume(pred: i32);
}

#[no_mangle]
fn spanic() -> ! {
    abort();
}

/// Reject the current execution with a verification failure.
///
/// In almost all circumstances, `report_error` should
/// be used instead because it generates an error message.
pub fn abort() -> ! {
    unsafe {
        __VERIFIER_error();
    }
}

/// Assume that condition `cond` is true
///
/// Any paths found must satisfy this assumption.
pub fn assume(pred: bool) {
    unsafe {
        __VERIFIER_assume(pred as i32);
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
    // panic!("RMC doesn't support replay.")
    false
}

/// Reject the current execution with a verification failure
/// and an error message.
pub fn report_error(message: &str) -> ! {
    // Mimic the format of klee_report_error
    // (We don't use klee_report_error because it is not
    // supported by the kleeRuntest library.)
    eprintln!("RMC: ERROR:{}", message);
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
        Some(msg) => eprintln!("VERIFIER_EXPECT: should_panic(expected = \"{}\")", msg),
    }
}

macro_rules! make_nondet {
    ($typ:ty) => {
        impl VerifierNonDet for $typ {
            fn verifier_nondet(self) -> Self {
                __nondet::<$typ>()
            }
        }
    };
}

make_nondet!(u8);
make_nondet!(u16);
make_nondet!(u32);
make_nondet!(u64);
make_nondet!(usize);

make_nondet!(i8);
make_nondet!(i16);
make_nondet!(i32);
make_nondet!(i64);
make_nondet!(isize);

make_nondet!(f32);
make_nondet!(f64);

macro_rules! make_nondet_ne_bytes {
    ($typ:ty) => {
        impl VerifierNonDet for $typ {
            fn verifier_nondet(self) -> Self {
                let mut bytes = vec![0u8; std::mem::size_of::<$typ>()];
                for i in 0..bytes.len() {
                    unsafe {
                        bytes[i] = __nondet::<u8>();
                    }
                }
                Self::from_ne_bytes(bytes[..].try_into().unwrap())
            }
        }
    };
}

make_nondet_ne_bytes!(u128);
make_nondet_ne_bytes!(i128);

impl VerifierNonDet for bool {
    fn verifier_nondet(self) -> Self {
        let c = __nondet::<u8>();
        assume(c == 0 || c == 1);
        c == 1
    }
}

/////////////////////////////////////////////////////////////////
// End
/////////////////////////////////////////////////////////////////
