////////////////////////////////////////////////////////////////
// Several variations on a theme to test failing variants
////////////////////////////////////////////////////////////////

#[cfg(feature = "verifier-klee")]
use crate::klee as verifier;

#[test]
fn t0() {
    let a = crate::abstract_value::<u32>();
    let b = verifier::abstract_value::<u32>();
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);
    if verifier::is_replay() { eprintln!("Test values: a = {}, b = {}", a, b) }

    let r = a*b;
    assert!(20 <= r && r <= 56);
}

#[cfg(test)]
#[test]
fn t1() {
    let a = verifier::abstract_value::<u32>();
    let b = verifier::abstract_value::<u32>();
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);
    let r = a*b;
    assert!(20 <= r && r <= 56);
}


#[cfg(test)]
#[test]
fn t2() {
    verifier::expect(Some("multiply with overflow"));
    let a = verifier::abstract_value::<u32>();
    let b = verifier::abstract_value::<u32>();
    let r = a*b;
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);
    assert!(20 <= r && r <= 56);
}

#[cfg(test)]
#[test]
fn t3() {
    verifier::expect(Some("assertion failed"));
    let a = verifier::abstract_value::<u32>();
    let b = verifier::abstract_value::<u32>();
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);
    let r = a*b;
    assert!(20 <= r && r < 56);
}

#[cfg(test)]
#[test]
fn t4() {
    verifier::expect(None);
    let a = verifier::abstract_value::<u32>();
    let b = verifier::abstract_value::<u32>();
    verifier::assume(4 <= a && a <= 7);
    verifier::assume(5 <= b && b <= 8);
    let r = a*b;
    assert!(20 <= r && r < 56);
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
