// Copyright 2020 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::default::Default;
use std::ffi::CString;

// Traits for creating symbolic/abstract values
#[cfg(feature = "verifier-klee")]
mod klee;
#[cfg(feature = "verifier-klee")]
pub use klee::*;

#[cfg(feature = "verifier-crux")]
pub extern crate crucible;
#[cfg(feature = "verifier-crux")]
mod crux;
#[cfg(feature = "verifier-crux")]
pub use crux::*;

#[cfg(feature = "verifier-rmc")]
mod rmc;
#[cfg(feature = "verifier-rmc")]
pub use rmc::*;

#[cfg(feature = "verifier-seahorn")]
mod seahorn;
#[cfg(feature = "verifier-seahorn")]
pub use seahorn::*;

/// Allocate a symbolic vector of bytes
pub fn verifier_nondet_bytes(n: usize) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::with_capacity(n);
    v.resize_with(n, || VerifierNonDet::verifier_nondet(0u8));
    return v;
}

/// Allocate a symbolic CString
pub fn verifier_nondet_cstring(size_excluding_null: usize) -> CString {
    let mut r = verifier_nondet_bytes(size_excluding_null + 1);
    for i in 0..size_excluding_null {
        assume(r[i] != 0u8);
    }
    r[size_excluding_null] = 0u8;
    unsafe { CString::from_vec_with_nul_unchecked(r) }
}

/// Allocate a symbolic ASCII String
/// (ASCII strings avoid the complexity of UTF-8)
pub fn verifier_nondet_ascii_string(n: usize) -> String {
    let r = verifier_nondet_bytes(n);
    for i in 0..n {
        assume(r[i] != 0u8);
        assume(r[i].is_ascii());
    }
    match String::from_utf8(r) {
        Ok(r) => r,
        Err(_) => reject(),
    }
}

impl<T: VerifierNonDet + Default> AbstractValue for T {
    fn abstract_value() -> Self {
        Self::verifier_nondet(Self::default())
    }
}

impl<T: VerifierNonDet + Default> Symbolic for T {
    fn symbolic(_desc: &'static str) -> Self {
        Self::verifier_nondet(Self::default())
    }
}

// Macros

#[macro_export]
macro_rules! assert {
    ($cond:expr,) => { $crate::verifier::assert!($cond) };
    ($cond:expr) => { $crate::verifier::assert!($cond, "assertion failed: {}", stringify!($cond)) };
    ($cond:expr, $($arg:tt)+) => {{
        if ! $cond {
            let message = format!($($arg)+);
            eprintln!("VERIFIER: panicked at '{}', {}:{}:{}",
                      message,
                      std::file!(), std::line!(), std::column!());
            $crate::verifier::abort();
        }
    }}
}

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {{
        let left = $left;
        let right = $right;
        $crate::verifier::assert!(
            left == right,
            "assertion failed: `(left == right)` \
             \n  left: `{:?}`,\n right: `{:?}`",
            left,
            right)
    }};
    ($left:expr, $right:expr, $fmt:tt $($arg:tt)*) => {{
        let left = $left;
        let right = $right;
        $crate::verifier::assert!(
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
        $crate::verifier::assert!(
            left != right,
            "assertion failed: `(left != right)` \
             \n  left: `{:?}`,\n right: `{:?}`",
            left,
            right)
    }};
    ($left:expr, $right:expr, $fmt:tt $($arg:tt)*) => {{
        let left = $left;
        let right = $right;
        $crate::verifier::assert!(
            left != right,
            concat!(
                "assertion failed: `(left != right)` \
                 \n  left: `{:?}`, \n right: `{:?}`: ", $fmt),
            left, right $($arg)*);
    }};
}

#[macro_export]
macro_rules! unreachable {
    () => {
        $crate::report_error("unreachable assertion was reached");
    };
}

pub use crate::assert;
pub use crate::assert_eq;
pub use crate::assert_ne;
pub use crate::unreachable;

#[cfg(feature = "verifier-klee")]
pub use crate::coherent;
