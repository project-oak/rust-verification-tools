// Copyright 2020 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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

// At the moment, the cargo-verify script does not support
// use of a separate test directory so, for now, we put
// the tests here.
#[cfg(test)]
mod tests;
