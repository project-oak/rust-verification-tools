////////////////////////////////////////////////////////////////
// Several variations on a theme to test failing variants
////////////////////////////////////////////////////////////////

use crate as verifier;

use crate::assert;

#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn t0() {
    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);

    #[cfg(not(any(feature = "verifier-crux", feature = "verifier-seahorn")))]
    if verifier::is_replay() { eprintln!("Test values: a = {}, b = {}", a, b) }

    let r = a*b;
    assert!(20 <= r && r <= 56);
}

#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn t1() {
    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);
    let r = a*b;
    assert!(20 <= r && r <= 56);
}

#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn t2() {
    #[cfg(not(feature = "verifier-crux"))]
    verifier::expect(Some("multiply with overflow"));

    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    let r = a*b;
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);
    assert!(20 <= r && r <= 56);
}

#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn t3() {
    #[cfg(not(feature = "verifier-crux"))]
    verifier::expect(Some("assertion failed"));

    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);
    let r = a*b;
    assert!(20 <= r && r < 56);
}

#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn t4() {
    #[cfg(not(feature = "verifier-crux"))]
    verifier::expect(None);

    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);
    let r = a*b;
    assert!(20 <= r && r < 56);
}

#[cfg_attr(not(feature = "verifier-crux"), test)]
#[cfg_attr(feature = "verifier-crux", crux_test)]
fn t5() {
    let a : u32 = verifier::AbstractValue::abstract_value();
    let b : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(a <= 1000000); // avoid overflow
    verifier::assume(b <= 1000000);
    verifier::assert_eq!(a + b, b + a);
    verifier::assert_ne!(a, a+1);
}

// KLEE-only test of get_concrete_value and is_symbolic
#[cfg(feature = "verifier-klee")]
#[test]
fn concrete1() {
    let a : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(a <= 100); // avoid overflow
    verifier::assert!(verifier::VerifierNonDet::is_symbolic(a));

    let b = verifier::VerifierNonDet::get_concrete_value(a);
    verifier::assert!(verifier::VerifierNonDet::is_symbolic(a));
    verifier::assert!(!verifier::VerifierNonDet::is_symbolic(b));
    verifier::assert!(b <= 100);

    // There is no expectation that each call to get_concrete_value
    // will produce a different result and, on KLEE, it can
    // produce the same result unless you add assumptions/branches
    // to avoid repetition.
    // This test is commented out because we don't want to insist
    // on one behavior or another.
    // let c = verifier::VerifierNonDet::get_concrete_value(a);
    // verifier::assert_eq!(b, c);
}

// KLEE-only test of concretize
#[cfg(feature = "verifier-klee")]
#[test]
fn concrete2() {
    let a : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(a <= 10); // limit number of distinct solutions
    verifier::assert!(verifier::VerifierNonDet::is_symbolic(a));

    let b = verifier::concretize(a);
    verifier::assert!(verifier::VerifierNonDet::is_symbolic(a));
    verifier::assert!(!verifier::VerifierNonDet::is_symbolic(b));
    verifier::assert!(b <= 10);
}

// KLEE-only test of sample
#[cfg(feature = "verifier-klee")]
#[test]
fn concrete3() {
    let a : u32 = verifier::AbstractValue::abstract_value();
    verifier::assume(a <= 1_000_000_000); // allow a huge number of solutions
    verifier::assert!(verifier::VerifierNonDet::is_symbolic(a));

    let b = verifier::sample(10, a); // consider only 10 of the possible solutions for a
    verifier::assert!(verifier::VerifierNonDet::is_symbolic(a));
    verifier::assert!(!verifier::VerifierNonDet::is_symbolic(b));
    verifier::assert!(b <= 1_000_000_000);
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
