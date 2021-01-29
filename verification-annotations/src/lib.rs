// Copyright 2020 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(cstring_from_vec_with_nul)]

// Traits for creating symbolic/abstract values
mod traits;
pub use crate::traits::*;

#[cfg(feature = "verifier-klee")]
mod klee;
#[cfg(feature = "verifier-klee")]
pub use crate::klee::*;

#[cfg(feature = "verifier-crux")]
pub extern crate crucible;
#[cfg(feature = "verifier-crux")]
mod crux;
#[cfg(feature = "verifier-crux")]
pub use crate::crux::*;

#[cfg(feature = "verifier-seahorn")]
mod seahorn;
#[cfg(feature = "verifier-seahorn")]
pub use crate::seahorn::*;

#[macro_export]
macro_rules! verifier_assert {
    ($cond:expr) => { $crate::assert!($cond); };
}

#[macro_export]
macro_rules! verifier_assume {
    ($cond:expr) => { $crate::assume!($cond); };
}

#[macro_export]
macro_rules! verifier_unreachable {
    () => { $crate::assert!(false, "unreachable assertion was reached"); };
}

// At the moment, the cargo-verify script does not support
// use of a separate test directory so, for now, we put
// the tests here.
#[cfg(test)]
mod tests;
