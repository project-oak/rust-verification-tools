/// Test the impact of path merging.
///
/// This is a scaling test to explore path explosions
/// using the canonical symbolic execution example of
/// a loop with multiple branches in it which results
/// in 2^N paths unless merging is used.
///
/// Running this test on KLEE with
///
///     cargo verify --backend=klee --tests -vv --backend-flags=--use-merge |& grep 'generated tests'
///
/// should result in
///
///       test_original: {"generated tests": 1024, "total instructions": 709566, "completed paths": 4093}
///       test_merged: {"generated tests": 1, "completed paths": 41, "total instructions": 11852}
///
/// Indicating that the original version suffers from a path explosion and generates
/// more tests, executes more instructions and explores more paths than the merged version.
use verification_annotations::prelude::*;

fn main() {
    println!("Hello, world!");
}

const N: usize = 10;

/// Test the impact of not using path merging.
///
/// The critical part of this example is the second loop
/// that contains multiple branches where the branches depend
/// on symbolic expressions.
///
/// On a classic symbolic execution tool, this will result in
/// 2^N paths.
#[test]
fn test_original() {
    // An array
    let mut a = [0u32; N];

    // Set each element of array to a symbolic value
    for i in &mut a {
        *i = u32::abstract_value();
    }

    // A loop containing two branches - this will cause a performance problem
    // for conventional symbolic execution.
    for x in &a {
        verifier::assume((5..10).contains(x) || (15..20).contains(x))
    }

    // A true assertion about an arbitrary element of the array
    verifier::assert!(a[3] < 20);
}

/// Test the impact of using path merging.
///
/// This test reduces the number of paths and instructions executed by merging
/// all the paths forked inside the loop.
///
/// There should be just one path when verified using KLEE with --use-merge
#[test]

/// The test depends on KLEE-specific features
#[cfg(feature = "verifier-klee")]
fn test_merged() {
    // An array
    let mut a = [0u32; N];

    // Set each element of array to a symbolic value
    for i in 0..a.len() {
        a[i] = u32::abstract_value();
    }

    // A loop containing two branches - this will cause a performance problem
    // for conventional symbolic execution.
    for x in a.iter() {
        verifier::coherent! {{
            verifier::assume((5..10).contains(x) || (15..20).contains(x))
        }}
    }

    // A true assertion about an arbitrary element of the array
    verifier::assert!(a[3] < 20);
}
