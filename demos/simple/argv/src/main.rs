#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

use std::env;

/// Simple test to confirm that it is possible to pass arguments
/// to Rust functions when verifying them.
///
/// Should pass:
///    `cargo verify . foo`
///    `cargo verify . foo foo`
///
/// Should fail:
///    `cargo verify .`
///    `cargo verify . foo bar`
fn main() {
    println!("{} args", env::args().len());
    verifier::assert!(env::args().len() >= 2);
    for a in env::args().skip(1) {
        println!("{}", a);
        verifier::assert!(a == "foo");
    }
}
