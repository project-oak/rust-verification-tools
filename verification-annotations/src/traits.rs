// Copyright 2020 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// There are three different styles of interface (that we are aware of):
// - symbolic(desc)     - takes a string argument 'desc' - optionally used in counterexamples
// - verifier_nondet(c) - takes a concrete argument 'c' that is ignored
// - abstract_value()   - no arguments
//
// For some (limited) amount of compatibility, we implement all three

use crate::assume;

/// Create a non-deterministic value with the same type as the argument
///
/// The argument does not influence the result of the function.
///
/// This is intended to be compatible with SMACK
pub trait VerifierNonDet {
    fn verifier_nondet(self) -> Self;
}

pub trait AbstractValue : Sized {
    /// Create an abstract value of type `Self`
    fn abstract_value() -> Self;

    /// Create an abstract value satisfying a predicate `F`
    fn abstract_where<F: FnOnce(&Self) -> bool>(f: F) -> Self {
        let x = Self::abstract_value();
        assume(f(&x));
        x
    }
}

/// Create a symbolic value of type `Self` and with
/// documentation name `desc`
///
/// This is intended to be compatible with Crux-MIR.
pub trait Symbolic: Sized {
    fn symbolic(desc: &'static str) -> Self;

    fn symbolic_where<F: FnOnce(&Self) -> bool>(desc: &'static str, f: F) -> Self {
        let x = Self::symbolic(desc);
        assume(f(&x));
        x
    }
}

/////////////////////////////////////////////////////////////////
// End
/////////////////////////////////////////////////////////////////
