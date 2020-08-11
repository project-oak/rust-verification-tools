// Copyright 2020 The Propverify authors
// Based on parts of Proptest which is Copyright 2017, 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

////////////////////////////////////////////////////////////////
// Test compatibility of the propverify crate with
// the original proptest crate.
//
// Some tests in this crate are based on proptest examples.
////////////////////////////////////////////////////////////////

#![allow(unused_imports)]

#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

mod collections;
mod compose;
mod dynamic;
mod enumeration;

// A simple test of the propverify/proptest library
proptest! {
    fn main(
        a in 4..8u32,
        b in 5..9u32)
    {
        let r = a*b;
        assert!(20 <= r && r <= 56);
    }
}

#[cfg(test)]
mod ranges {
    use super::*;

    proptest! {
        #[test]
        #[should_panic(expected = "overflow")]
        fn i64_abs_is_never_negative(a in (std::i64::MIN .. std::i64::MAX)) {
            // This actually fails if a == i64::MIN, but randomly picking one
            // specific value out of 2⁶⁴ is overwhelmingly unlikely.
            assert!(a.abs() >= 0);
        }
    }

    proptest! {
        #[test]
        fn range1(
            a in (std::i32::MIN/2 .. std::i32::MAX/2),
            b in (std::i32::MIN/2 .. std::i32::MAX/2),
        ) {
            assert_eq!(a + b, b + a);
        }
    }

    proptest! {
        #[test]
        fn range2(a in 0..10u32, b in 10..20u32) {
            assert_ne!(a, b)
        }
    }

    proptest! {
        #[test]
        #[should_panic(expected = "attempt to add with overflow")]
        fn overflow1(
            a in 0 .. std::i32::MAX,
            b in 0 .. std::i32::MAX,
        ) {
            // Note that overflow tests have to be fairly unsubtle or
            // fuzzing can miss it. eg catching "a+1 overflows" is
            // unlikely beyond 8-bit ints.
            assert_eq!(a + b, b + a);
        }
    }
}

#[cfg(test)]
mod tuples {
    use super::*;

    proptest! {
        #[test]
        fn tuple2((a, b) in (0u32.., 0u32..)) {
            assert_eq!((a <= b), (b >= a));
        }
    }
}

#[cfg(test)]
mod prop {
    use super::*;

    proptest! {
        #[test]
        fn filter_map(a in (0u32..1000).prop_filter_map("%2", |x| if x % 2 == 0 { Some(x*2) } else { None })) {
            assert!(a % 4 == 0);
        }
    }

    proptest! {
        #[test]
        #[should_panic(expected = "assertion failed")]
        fn filter_map_fail1(a in (0u32..1000).prop_filter_map("%2", |x| if x % 2 == 0 { Some(x*2) } else { None })) {
            assert!(a % 8 == 0);
        }
    }

    proptest! {
        #[test]
        fn filter(a in (0..).prop_filter("%4", |x| x % 4 == 0)) {
            assert!(a % 2 == 0);
        }
    }

    proptest! {
        #[test]
        fn flat_map((a, b) in (1..65536).prop_flat_map(|a| (Just(a), 0..a))) {
            assert!(a > b);
        }
    }

    proptest! {
        #[test]
        #[should_panic(expected = "assertion failed")]
        fn flat_map_fail1((a, b) in (1..65536).prop_flat_map(|a| (Just(a), 0..a))) {
            assert!(a <= b);
        }
    }

    proptest! {
        #[test]
        fn ind_flat_map((a, b) in (1..65536).prop_ind_flat_map(|a| (Just(a), 0..a))) {
            assert!(a > b);
        }
    }

    proptest! {
        #[test]
        fn ind_flat_map2((a, b) in (1..65536).prop_ind_flat_map2(|a| (0..a))) {
            assert!(a > b);
        }
    }

    proptest! {
        #[test]
        #[should_panic(expected = "assertion failed")]
        fn ind_flat_map2_fail1((a, b) in (1..65536).prop_ind_flat_map2(|a| (0..a))) {
            assert!(a <= b);
        }
    }

    proptest! {
        #[test]
        fn map_into(a in (0u8..).prop_map_into::<u32>()) {
            assert!(a < 256);
        }
    }

    proptest! {
        #[test]
        fn map(a in (0..10i32).prop_map(|x| x+50)) {
            assert!(a >= 50);
        }
    }

    proptest! {
        #[test]
        #[should_panic(expected = "assertion failed")]
        fn map_fail1(a in (0..10i32).prop_map(|x| x+50)) {
            assert!(a < 10);
        }
    }

    proptest! {
        #[test]
        #[should_panic(expected = "assertion failed")]
        fn map_into_fail1(a in (0u16..).prop_map_into::<u32>()) {
            assert!(a < 256);
        }
    }
}

#[cfg(test)]
mod union {
    use super::*;

    proptest! {
        #[test]
        fn union1(v in prop_oneof![0..10u32, 30u32..]) {
            assert!((0..10).contains(&v) || (30..).contains(&v));
        }
    }

    proptest! {
        #[test]
        #[should_panic(expected = "assertion failed")]
        fn union_fail1(a in (0..10i32).prop_union(20..30i32)) {
            assert!(a != 25);
        }
    }
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
