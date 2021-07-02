// Copyright 2020 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Note: this file is a Frankenstein's monster that combines all the
// source files from this crate into a single file to allow us to
// run RMC on (some approximation of) the crate.
//
// As soon as it is possible to use RMC with cargo, any necessary changes
// should be backported to src/verifier/rmc.rs and this file should
// be deleted.
//
// At present, RMC reports the following
//
//    ** 5 of 26 failed (4 iterations)
//    VERIFICATION FAILED
//
// with the usual cause of failure being integer overflow (or + or *).


// There are three different styles of interface (that we are aware of):
// - symbolic(desc)     - takes a string argument 'desc' - optionally used in counterexamples
// - verifier_nondet(c) - takes a concrete argument 'c' that is ignored
// - abstract_value()   - no arguments
//
// For some (limited) amount of compatibility, we implement all three

#![feature(cstring_from_vec_with_nul)]

use std::default::Default;
use std::ffi::CString;

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

#[macro_export]
macro_rules! unreachable {
    () => {
        $crate::report_error("unreachable assertion was reached");
    };
}

#[cfg(feature = "verifier-klee")]
pub use crate::coherent;

/// Create a non-deterministic value with the same type as the argument
///
/// The argument does not influence the result of the function.
///
/// This is intended to be compatible with SMACK
pub trait VerifierNonDet {
    fn verifier_nondet(self) -> Self;

    #[cfg(feature = "verifier-klee")]
    /// Obtain a concrete value satisfying the constraints
    /// currently in force for the expression.
    ///
    /// Not guaranteed to produce different (or the same) value
    /// if called repeatedly.
    /// (Use assumptions or if-statements to produce different results
    /// each time.)
    ///
    /// This function may not be implementable with other
    /// verifiers so it should be used with caution.
    fn get_concrete_value(x: Self) -> Self;

    #[cfg(feature = "verifier-klee")]
    /// Test whether a value is concrete or symbolic
    ///
    /// Values are guaranteed to be concrete if they are derived
    /// from concrete values, literal constants or
    /// calls to `get_concrete_value`.
    fn is_symbolic(x: Self) -> bool;
}

pub trait AbstractValue: Sized {
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

// Traits for creating symbolic/abstract values
pub trait UnwrapOrReject {
    type Wrapped;
    fn unwrap_or_reject(self) -> Self::Wrapped;
}

impl<T, E> UnwrapOrReject for Result<T, E> {
    type Wrapped = T;
    fn unwrap_or_reject(self) -> Self::Wrapped {
        match self {
            Ok(x) => x,
            Err(_) => reject(),
        }
    }
}

impl<T> UnwrapOrReject for Option<T> {
    type Wrapped = T;
    fn unwrap_or_reject(self) -> Self::Wrapped {
        match self {
            Some(x) => x,
            None => reject(),
        }
    }
}

/////////////////////////////////////////////////////////////////
// FFI wrapper for RMC model checker
/////////////////////////////////////////////////////////////////

use std::convert::TryInto;

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
				fn __nondet() -> $typ {
					unimplemented!()
				}

                __nondet()
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
                        bytes[i] = u8::verifier_nondet(0);
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
        let c = u8::verifier_nondet(0);
        assume(c == 0 || c == 1);
        c == 1
    }
}

////////////////////////////////////////////////////////////////
// Several variations on a theme to test failing variants
////////////////////////////////////////////////////////////////

// #[cfg_attr(not(feature = "verifier-crux"), test)]
// #[cfg_attr(feature = "verifier-crux", crux_test)]
fn t0() {
    let a: u32 = AbstractValue::abstract_value();
    let b: u32 = AbstractValue::abstract_value();
    assume(4 <= a && a <= 7);
    assume(5 <= b && b <= 8);

    #[cfg(not(any(feature = "verifier-crux", feature = "verifier-seahorn", feature = "verifier-cbmc")))]
    if is_replay() {
        eprintln!("Test values: a = {}, b = {}", a, b)
    }

    let r = a * b;
    assert!(20 <= r && r <= 56);
}

// #[cfg_attr(not(feature = "verifier-crux"), test)]
// #[cfg_attr(feature = "verifier-crux", crux_test)]
fn t1() {
    let a: u32 = AbstractValue::abstract_value();
    let b: u32 = AbstractValue::abstract_value();
    assume(4 <= a && a <= 7);
    assume(5 <= b && b <= 8);
    let r = a * b;
    assert!(20 <= r && r <= 56);
}

// #[cfg_attr(not(feature = "verifier-crux"), test)]
// #[cfg_attr(feature = "verifier-crux", crux_test)]
fn t2() {
    #[cfg(not(feature = "verifier-crux"))]
    expect(Some("multiply with overflow"));

    let a: u32 = AbstractValue::abstract_value();
    let b: u32 = AbstractValue::abstract_value();
    let r = a * b;
    assume(4 <= a && a <= 7);
    assume(5 <= b && b <= 8);
    assert!(20 <= r && r <= 56);
}

// #[cfg_attr(not(feature = "verifier-crux"), test)]
// #[cfg_attr(feature = "verifier-crux", crux_test)]
fn t3() {
    #[cfg(not(feature = "verifier-crux"))]
    expect(Some("assertion failed"));

    let a: u8 = AbstractValue::abstract_value();
    let b: u8 = AbstractValue::abstract_value();
    assume(4 <= a && a <= 7);
    assume(5 <= b && b <= 8);
    let r = (a as u32) * (b as u32);
    assert!(20 <= r && r < 56);
}

// #[cfg_attr(not(feature = "verifier-crux"), test)]
// #[cfg_attr(feature = "verifier-crux", crux_test)]
fn t4() {
    #[cfg(not(feature = "verifier-crux"))]
    expect(None);

    let a: u32 = AbstractValue::abstract_value();
    let b: u32 = AbstractValue::abstract_value();
    assume(4 <= a && a <= 7);
    assume(5 <= b && b <= 8);
    let r = a * b;
    assert!(20 <= r && r < 56);
}

// #[cfg_attr(not(feature = "verifier-crux"), test)]
// #[cfg_attr(feature = "verifier-crux", crux_test)]
fn t5() {
    let a: u32 = AbstractValue::abstract_value();
    let b: u32 = AbstractValue::abstract_value();
    assume(a <= 1000000); // avoid overflow
    assume(b <= 1000000);
    assert_eq!(a + b, b + a);
    assert_ne!(a, a + 1);
}

// KLEE-only test of get_concrete_value and is_symbolic
#[cfg(feature = "verifier-klee")]
#[test]
fn concrete1() {
    let a: u32 = AbstractValue::abstract_value();
    assume(a <= 100); // avoid overflow
    assert!(VerifierNonDet::is_symbolic(a));

    let b = VerifierNonDet::get_concrete_value(a);
    assert!(VerifierNonDet::is_symbolic(a));
    assert!(!VerifierNonDet::is_symbolic(b));
    assert!(b <= 100);

    // There is no expectation that each call to get_concrete_value
    // will produce a different result and, on KLEE, it can
    // produce the same result unless you add assumptions/branches
    // to avoid repetition.
    // This test is commented out because we don't want to insist
    // on one behavior or another.
    // let c = VerifierNonDet::get_concrete_value(a);
    // assert_eq!(b, c);
}

// KLEE-only test of concretize
#[cfg(feature = "verifier-klee")]
#[test]
fn concrete2() {
    let a: u32 = AbstractValue::abstract_value();
    assume(a <= 10); // limit number of distinct solutions
    assert!(VerifierNonDet::is_symbolic(a));

    let b = concretize(a);
    assert!(VerifierNonDet::is_symbolic(a));
    assert!(!VerifierNonDet::is_symbolic(b));
    assert!(b <= 10);
}

// KLEE-only test of sample
#[cfg(feature = "verifier-klee")]
#[test]
fn concrete3() {
    let a: u32 = AbstractValue::abstract_value();
    assume(a <= 1_000_000_000); // allow a huge number of solutions
    assert!(VerifierNonDet::is_symbolic(a));

    let b = sample(10, a); // consider only 10 of the possible solutions for a
    assert!(VerifierNonDet::is_symbolic(a));
    assert!(!VerifierNonDet::is_symbolic(b));
    assert!(b <= 1_000_000_000);
}

/// Test of verifier_nondet_bytes
#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn bytes() {
    let a = verifier_nondet_bytes(8);
    for i in &a {
        assume(*i == 42);
    }
    if is_replay() {
        println!("{:?}", a);
    }
    assert_eq!(a.len(), 8);
    assert_ne!(a[2], 0u8);
    assert_eq!(a[3], 42u8);
}

/// Test of verifier_nondet_cstring
#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn cstring() {
    let a = verifier_nondet_cstring(8);

    if is_replay() {
        println!("{:?}", a);
    }

    // force string to be plain ASCII - to keep things simple
    for i in a.as_bytes() {
        // note: this code suffers from a path explosion and
        // would benefit from using coherent!
        // We will not do that though because the goal is to
        // test this feature in isolation.
        assume(i.is_ascii_alphabetic());
    }

    for i in a.as_bytes() {
        assert!(i.is_ascii());
        // this assertion would fail
        // assert!(i.is_ascii_digit());
    }
}

/// Test of verifier_nondet_ascii_string
#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn string_ok() {
    let a = verifier_nondet_ascii_string(6);

    if is_replay() {
        println!("{:?}", a);
    }

    // force string to be a legal int
    for i in a.as_bytes() {
        assume(('0'..='3').contains(&(*i as char)))
    }

    let i: u32 = a.parse().unwrap();
    assert!(i <= 333_333);
}

/// Test of  verifier_nondet_ascii_stringg
#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn string_should_fail() {
    expect(Some("assertion failed"));
    let a = verifier_nondet_ascii_string(6);

    if is_replay() {
        println!("{:?}", a);
    }

    // force string to be a legal int
    for i in a.as_bytes() {
        assume(('0'..='3').contains(&(*i as char)))
    }

    let i: u32 = a.parse().unwrap();
    assert!(i <= 222_222);
}

fn main() {
    match u8::abstract_value() {
        0 => t0(),
        1 => t1(),
        2 => t2(),
        3 => t3(),
        4 => t4(),
        5 => t5(),
        _ => ()
    }
}

/////////////////////////////////////////////////////////////////
// End
/////////////////////////////////////////////////////////////////
