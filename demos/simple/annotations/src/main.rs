use verification_annotations::prelude::*;

#[cfg_attr(feature = "verifier-crux", crux_test)]
#[cfg_attr(not(feature = "verifier-crux"), test)]
fn t1() {
    let a = u32::abstract_value();
    let b = u32::abstract_value();
    verifier::assume(1 <= a && a <= 1000);
    verifier::assume(1 <= b && b <= 1000);
    if verifier::is_replay() {
        eprintln!("Test values: a = {}, b = {}", a, b);
    }
    let r = a * b;
    verifier::assert!(1 <= r && r < 1000000);
}
