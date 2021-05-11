/// Test the use of regular expressions
///
/// This test checks three things
///
/// 1. Feature support for hand-vectorized code that
///    includes a "slow path" if vector instructions are not
///    available.
///    (We are testing that the "fast path" is ignored.)
///
/// 2. Feature support for an aligned memory allocator.
///    (This is the part of the hand-vectorized code that we
///    cannot disable.)
///
/// 3. A path explosion in the handling of regular expressions.
///    At present, this explosion is not fixed and we keep the
///    strings very short to keep execution time reasonable.
use regex::Regex;
use verification_annotations::verifier;

fn main() {
    println!("Hello, world!");
}

const N: usize = 2;

/// Test use of regular expressions
#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn regex_ok() {
    let a = verifier::verifier_nondet_ascii_string(N);

    if verifier::is_replay() {
        println!("Value a = {:?}", a);
    }

    verifier::assume(Regex::new(r"[0-1]{2}").unwrap().is_match(&a));

    let i: u32 = a.parse().unwrap();
    verifier::assert!(i <= 11);
}

/// Test use of regular expressions
#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn regex_should_fail() {
    verifier::expect(Some("assertion failed"));
    let a = verifier::verifier_nondet_ascii_string(N);

    if verifier::is_replay() {
        println!("Value a = {:?}", a);
    }

    verifier::assume(Regex::new(r"[0-1]{2}").unwrap().is_match(&a));

    let i: u32 = a.parse().unwrap();
    verifier::assert!(i < 11);
}
