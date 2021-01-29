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

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
