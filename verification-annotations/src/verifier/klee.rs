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

#[link(name = "kleeRuntest")]
extern "C" {
    fn klee_make_symbolic(data: *mut u8, length: usize, name: *const i8);
    fn klee_assume(cond: usize);
    fn klee_abort() -> !;
    fn klee_silent_exit(_ignored: u32) -> !;
    fn klee_is_replay() -> i32;
    fn klee_open_merge();
    fn klee_close_merge();
    fn klee_get_value_i32(x: i32) -> i32;
    fn klee_get_value_i64(x: i64) -> i64;
    fn klee_get_value_f(x: f32) -> f32;
    fn klee_get_value_d(x: f64) -> f64;
    fn klee_is_symbolic(x: usize) -> i32;
}

unsafe fn get_value_i8(x: i8) -> i8 {
    klee_get_value_i32(x as i32) as i8
}

unsafe fn get_value_u8(x: u8) -> u8 {
    klee_get_value_i32(x as i32) as u8
}

unsafe fn get_value_i16(x: i16) -> i16 {
    klee_get_value_i32(x as i32) as i16
}

unsafe fn get_value_u16(x: u16) -> u16 {
    klee_get_value_i32(x as i32) as u16
}

unsafe fn get_value_u32(x: u32) -> u32 {
    klee_get_value_i32(x as i32) as u32
}

unsafe fn get_value_u64(x: u64) -> u64 {
    klee_get_value_i64(x as i64) as u64
}

unsafe fn get_value_i128(x: i128) -> i128 {
    get_value_u128(x as u128) as i128
}

unsafe fn get_value_u128(x: u128) -> u128 {
    let hi = (x >> 64) as u64;
    let lo = x as u64;
    let hi = klee_get_value_i64(hi as i64) as u64;
    let lo = klee_get_value_i64(lo as i64) as u64;
    ((hi as u128) << 64) | (lo as u128)
}

unsafe fn get_value_isize(x: isize) -> isize {
    klee_get_value_i64(x as i64) as isize
}

unsafe fn get_value_usize(x: usize) -> usize {
    klee_get_value_i64(x as i64) as usize
}

/// Create instance for any type consisting of contiguous memory
/// where all bit-patterns are legal values of the type.
macro_rules! make_verifier_nondet {
    ($typ:ident, $get_value:ident) => {
        impl VerifierNonDet for $typ {
            fn verifier_nondet(self) -> Self {
                let mut r = self;
                unsafe {
                    let data: *mut u8 = &mut r as *mut $typ as *mut u8;
                    let length = core::mem::size_of::<$typ>();
                    let null = 0 as *const i8;
                    klee_make_symbolic(data, length, null)
                }
                return r;
            }

            fn get_concrete_value(x: Self) -> Self {
                unsafe { $get_value(x) }
            }

            fn is_symbolic(x: Self) -> bool {
                unsafe { klee_is_symbolic(x as usize) != 0 }
            }
        }
    };
}

make_verifier_nondet!(u8, get_value_u8);
make_verifier_nondet!(u16, get_value_u16);
make_verifier_nondet!(u32, get_value_u32);
make_verifier_nondet!(u64, get_value_u64);
make_verifier_nondet!(u128, get_value_u128);
make_verifier_nondet!(usize, get_value_usize);

make_verifier_nondet!(i8, get_value_i8);
make_verifier_nondet!(i16, get_value_i16);
make_verifier_nondet!(i32, klee_get_value_i32);
make_verifier_nondet!(i64, klee_get_value_i64);
make_verifier_nondet!(i128, get_value_i128);
make_verifier_nondet!(isize, get_value_isize);

make_verifier_nondet!(f32, klee_get_value_f);
make_verifier_nondet!(f64, klee_get_value_d);

impl VerifierNonDet for bool {
    fn verifier_nondet(self) -> Self {
        let c = VerifierNonDet::verifier_nondet(0u8);
        assume(c == 0 || c == 1);
        c == 1
    }

    fn get_concrete_value(x: Self) -> Self {
        unsafe { klee_get_value_i32(x as i32) != 0 }
    }

    fn is_symbolic(x: Self) -> bool {
        unsafe { klee_is_symbolic(x as usize) != 0 }
    }
}

/// Assume that condition `cond` is true
///
/// Any paths found must satisfy this assumption.
pub fn assume(cond: bool) {
    unsafe { klee_assume(if cond { 1 } else { 0 }) }
}

/// Reject the current execution with a verification failure.
///
/// In almost all circumstances, `report_error` should
/// be used instead because it generates an error message.
pub fn abort() -> ! {
    unsafe { klee_abort() }
}

/// Reject the current execution path with a verification success.
/// This is equivalent to `assume(false)`
/// and the opposite of `report_error(...)`.
///
/// Typical usage is in generating symbolic values when the value
/// does not meet some criteria.
pub fn reject() -> ! {
    unsafe { klee_silent_exit(0) }
}

/// Detect whether the program is being run symbolically in KLEE
/// or being replayed using the kleeRuntest runtime.
///
/// This is used to decide whether to display the values of
/// variables that may be either symbolic or concrete.
pub fn is_replay() -> bool {
    unsafe { klee_is_replay() != 0 }
}

/// Open a merge block
///
/// Should be paired with `close_merge`
pub fn open_merge() {
    // safe because it only affects KLEE scheduling
    unsafe {
        klee_open_merge();
    }
}

/// Close a merge block
///
/// Should be paired with `open_merge`
pub fn close_merge() {
    // safe because it only affects KLEE scheduling
    unsafe {
        klee_close_merge();
    }
}

/// Coherent blocks don't fork execution during verification.
///
/// This will only take effect if executed with the
/// KLEE command line flags `--use-merge` and, optionally,
/// `--use-incomplete-merge`.
///
/// This might reduce the number of instructions that KLEE explores
/// because there are less forks.
/// This might also make evaluation of the symbolic constraints
/// more expensive because of state merging.
///
/// Caveats:
/// - Branches out of the middle of `$body` such as return, etc. will not be merged.
///   If this is a problem, you should use open_merge/close_merge explicitly.
///
/// - If the body performs memory allocation, merging cannot happen (KLEE limitation).
#[macro_export]
macro_rules! coherent {
    ( $body:block ) => {
        $crate::verifier::open_merge();
        $body;
        $crate::verifier::close_merge();
    };
}

/// Exhaustively enumerate all possible concrete values for `x`.
///
/// If there are a finite number of possible values for `x`,
/// this terminates because get_concrete_value terminates this path
/// if there are no further solutions.
pub fn concretize<T>(x: T) -> T
where
    T: VerifierNonDet + Eq + Copy,
{
    loop {
        let s = T::get_concrete_value(x);
        if s == x {
            return s;
        }
    }
}

/// Generate `samples` paths each of which explores a single, distinct
/// concrete value for `x` which is expected to be symbolic.
///
/// In most cases, this results in an incomplete exploration because there may
/// be more possible solutions than we explore.
pub fn sample<T>(samples: usize, x: T) -> T
where
    T: VerifierNonDet + Eq + Copy,
{
    for _i in 0..samples - 1 {
        let s = T::get_concrete_value(x);
        if s == x {
            return s;
        }
    }
    T::get_concrete_value(x)
}

#[cfg(feature = "std")]
/// Reject the current execution with a verification failure
/// and an error message.
pub fn report_error(message: &str) -> ! {
    // Mimic the format of klee_report_error
    // (We don't use klee_report_error because it is not
    // supported by the kleeRuntest library.)
    eprintln!("KLEE: ERROR:{}", message);
    abort();
}

#[cfg(feature = "std")]
/// Declare that failure is the expected behaviour
pub fn expect_raw(msg: &str) {
    eprintln!("VERIFIER_EXPECT: {}", msg)
}

#[cfg(feature = "std")]
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
