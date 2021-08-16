use verification_annotations::prelude::*;

#[macro_use]
extern crate mirai_annotations;

pub fn main() {
    match abstract_value!(0) {
        // 0 => test0(),
        // 1 => test00(),
        // 2 => test1(),
        3 => test2(),
        // 4 => test3(),
        _ => ()
    }
}

// // BAD
// // This is how we usually write the simple test. The assertion should be hit,
// // but MIRAI does not report anything.
// fn test0() {
//     let a: u32 = u32::abstract_value();
//     let b: u32 = u32::abstract_value();
//     verifier::assume(1 <= a && a <= 10);
//     verifier::assume(1 <= b && b <= 10);
//     let r = a * b;
//     verifier::assert!(1 <= r && r < 100);
// }

// // GOOD
// // If we write the same test using MIRAI's native annotations, MIRAI reports the
// // second assertion.
// fn test00() {
//     let a: u32 = abstract_value!(1);
//     let b: u32 = abstract_value!(1);
//     assume!(1 <= a && a <= 10);
//     assume!(1 <= b && b <= 10);
//     let r = a * b;
//     verify!(1 <= r && r <= 100);
//     verify!(1 <= r && r < 100);
// }

// // BAD
// fn test1() {
//     let a = u32::abstract_value();
//     // MIRAI: "warning: assumption is provably false and it will be ignored"
//     assume!(1 <= a && a <= 10);

//     let b = test1_helper();
//     // No warning here
//     assume!(1 <= b && b <= 10);
// }

// fn test1_helper() -> u32 { abstract_value!(1) }

// BAD
// `verifier::assume` has no effect.
fn test2() {
    let a: u32 = abstract_value!(1);
    let b: u32 = abstract_value!(1);
    // verifier::
    assume2(1 <= a && a <= 10);
    // verifier::
    assume2(1 <= b && b <= 10);
    let r = a * b;
    verify!(1 <= r && r <= 100);
}

// fn assume(_cond: bool) {
//     // assume!(cond);
//     postcondition!(_cond);
// }


// // BAD
// // `verifier::assert!` is ignored.
// fn test3() {
//     let a: u32 = abstract_value!(1);
//     let b: u32 = abstract_value!(1);
//     assume!(1 <= a && a <= 10);
//     assume!(1 <= b && b <= 10);
//     let r = a * b;
//     verifier::assert!(1 <= r && r < 100);
//     test3_helper(r);
//     assert(1 <= r && r <= 100);
// }

// // BAD
// // `verify!` is hit because contracts are not inferred?
// fn test3_helper(r: u32) {
//     verify!(1 <= r && r <= 100);
// }

// fn assert(cond : bool) {
//     precondition!(cond);
//     if !cond {
//         abort();
//     }
// }

// fn abort() {
//     verify!(false);
// }
