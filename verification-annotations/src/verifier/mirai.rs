// Copyright 2020 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use crate::traits::*;

/// Create instance for any type consisting of contiguous memory
/// where all bit-patterns are legal values of the type.
macro_rules! make_verifier_nondet {
    ($typ:ident) => {
        impl VerifierNonDet for $typ {
            fn verifier_nondet(self) -> Self {
                abstract_value!(self)
            }

            // fn get_concrete_value(x: Self) -> Self {
            // }

            // fn is_symbolic(x: Self) -> bool {
            // }
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

make_verifier_nondet!(bool);

/// Assume that condition `cond` is true
///
/// Any paths found must satisfy this assumption.
pub fn assume(cond: bool) {
    precondition!(true);
    postcondition!(cond);
    let a : u32 = abstract_value!(0);
    let b : u32 = abstract_value!(0);
    let c : u32 = a + b;
    verify!(c > a);
    assume!(cond);
}

/// Reject the current execution with a verification failure.
///
/// In almost all circumstances, `report_error` should
/// be used instead because it generates an error message.
pub fn abort() {
    precondition!(false);
    verify!(false);
    // panic!("Unreachable, should have been aborted!");
}

/// Reject the current execution path with a verification success.
/// This is equivalent to `assume(false)`
/// and the opposite of `report_error(...)`.
///
/// Typical usage is in generating symbolic values when the value
/// does not meet some criteria.
pub fn reject() -> ! {
    assume!(false);
    panic!("Unreachable, should have been rejected!");
}

/// Detect whether the program is being run symbolically in KLEE
/// or being replayed using the kleeRuntest runtime.
///
/// This is used to decide whether to display the values of
/// variables that may be either symbolic or concrete.
pub fn is_replay() -> bool {
    false
}




// /// Open a merge block
// ///
// /// Should be paired with `close_merge`
// pub fn open_merge() {
//     // safe because it only affects KLEE scheduling
//     unsafe {
//         klee_open_merge();
//     }
// }

// /// Close a merge block
// ///
// /// Should be paired with `open_merge`
// pub fn close_merge() {
//     // safe because it only affects KLEE scheduling
//     unsafe {
//         klee_close_merge();
//     }
// }

// /// Coherent blocks don't fork execution during verification.
// ///
// /// This will only take effect if executed with the
// /// KLEE command line flags `--use-merge` and, optionally,
// /// `--use-incomplete-merge`.
// ///
// /// This might reduce the number of instructions that KLEE explores
// /// because there are less forks.
// /// This might also make evaluation of the symbolic constraints
// /// more expensive because of state merging.
// ///
// /// Caveats:
// /// - Branches out of the middle of `$body` such as return, etc. will not be merged.
// ///   If this is a problem, you should use open_merge/close_merge explicitly.
// ///
// /// - If the body performs memory allocation, merging cannot happen (KLEE limitation).
// #[macro_export]
// macro_rules! coherent {
//     ( $body:block ) => {
//         $crate::verifier::open_merge();
//         $body;
//         $crate::verifier::close_merge();
//     };
// }

// /// Exhaustively enumerate all possible concrete values for `x`.
// ///
// /// If there are a finite number of possible values for `x`,
// /// this terminates because get_concrete_value terminates this path
// /// if there are no further solutions.
// pub fn concretize<T>(x: T) -> T
// where
//     T: VerifierNonDet + Eq + Copy,
// {
//     loop {
//         let s = T::get_concrete_value(x);
//         if s == x {
//             return s;
//         }
//     }
// }

// /// Generate `samples` paths each of which explores a single, distinct
// /// concrete value for `x` which is expected to be symbolic.
// ///
// /// In most cases, this results in an incomplete exploration because there may
// /// be more possible solutions than we explore.
// pub fn sample<T>(samples: usize, x: T) -> T
// where
//     T: VerifierNonDet + Eq + Copy,
// {
//     for _i in 0..samples - 1 {
//         let s = T::get_concrete_value(x);
//         if s == x {
//             return s;
//         }
//     }
//     T::get_concrete_value(x)
// }

/// Reject the current execution with a verification failure
/// and an error message.
pub fn report_error(message: &str) {
    // Mimic the format of klee_report_error
    // (We don't use klee_report_error because it is not
    // supported by the kleeRuntest library.)
    eprintln!("MIRAI: ERROR:{}", message);
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

/////////////////////////////////////////////////////////////////
// End
/////////////////////////////////////////////////////////////////
