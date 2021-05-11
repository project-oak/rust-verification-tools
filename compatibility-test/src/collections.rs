// Copyright 2020 The Propverify authors
// Based on parts of Proptest which is Copyright 2017, 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

////////////////////////////////////////////////////////////////
// Proptest-based tests to check collection support
////////////////////////////////////////////////////////////////

#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

proptest! {
    #[test]
    fn binary_heap(v in prop::collection::binary_heap(0..100u32, 5)) {
        assert!(v.len() == 5);
        for x in v.iter() {
            assert!(*x < 100);
        }

        // check first element larger than rest
        let mut v1 = v;
        let x0 = v1.pop().unwrap();
        for x in v1.iter() {
            assert!(*x <= x0);
        }
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn binary_heap_fail1(v in prop::collection::binary_heap(0..100u32, 5)) {
        for x in v.iter() {
            assert!(*x < 10);
        }
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn binary_heap_fail2(v in prop::collection::binary_heap(0..100u32, 5)) {
        // check first element smaller than rest
        let mut v1 = v;
        let x0 = v1.pop().unwrap();
        for x in v1.iter() {
            assert!(*x < x0);
        }
    }
}

proptest! {
    #[test]
    fn btree_map(v in prop::collection::btree_map(-5..5i32, 10..20u32, 5)) {

        // Note that key collisions may reduce the number of entries
        // so the following assertion will fail.
        // assert!(v.len() == 5);
        assert!(v.len() <= 5);

        for (key, value) in v.iter() {
            assert!((-5..5i32).contains(key));
            assert!((*value) > 5);
        }
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn btree_map_fail1(v in prop::collection::btree_map(-5..5i32, 10..20u32, 5)) {
        for (key, _) in v.iter() {
            assert!((0..5i32).contains(key));
        }
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn btree_map_fail2(v in prop::collection::btree_map(-5..5i32, 10..20u32, 5)) {
        for (_, value) in v.iter() {
            assert!((*value) > 15);
        }
    }
}

proptest! {
    #[test]
    fn btree_set(v in prop::collection::btree_set(-100..100i32, 5)) {

        // Note that key collisions may reduce the number of entries
        // so the following assertion will fail.
        // assert!(v.len() == 5);
        assert!(v.len() <= 5);

        for x in v.iter() {
            assert!((-100..100i32).contains(x));
        }
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn btree_set_fail1(v in prop::collection::btree_set(-100..100i32, 5)) {
        for x in v.iter() {
            assert!((0..100i32).contains(x));
        }
    }
}

proptest! {
    #[test]
    fn linked_list(v in prop::collection::linked_list(0..10u32, 5)) {
        assert!(v.len() == 5);
        for x in &v {
            assert!(*x < 10);
        }
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn linked_list_fail1(v in prop::collection::linked_list(0..10u32, 5)) {
        for x in &v {
            assert!(*x < 5);
        }
    }
}

proptest! {
    #[test]
    fn vec_deque(v in prop::collection::vec_deque(0..10u32, 5)) {
        assert!(v.len() == 5);
        for x in &v {
            assert!(*x < 10);
        }
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn vec_deque_fail1(v in prop::collection::vec_deque(0..10u32, 5)) {
        for x in &v {
            assert!(*x < 5);
        }
    }
}

proptest! {
    #[test]
    fn vec(v in prop::collection::vec(0..10u32, 5)) {
        assert!(v.len() == 5);
        for x in &v {
            assert!(*x < 10);
        }
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn vec_fail1(v in prop::collection::vec(0..10u32, 5)) {
        for x in &v {
            assert!(*x < 5);
        }
    }
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
