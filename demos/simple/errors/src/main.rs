/// Test detection of various types of error

#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

proptest! {
    // #[should_panic(expected = "overflow")]
    #[cfg_attr(feature="verifier-crux", crux_test)]
    #[cfg_attr(not(feature="verifier-crux"), test)]
    fn overflow_should_fail(a: u32) {
        // Normally, this could fail due to wraparound but we verify
        // with overflow checks enabled so it cannot fail.
        assert!(a+1 >= a);
    }
}

proptest! {
    // #[should_panic(expected = "index out of bounds")]
    #[allow(unconditional_panic)]
    #[cfg_attr(feature="verifier-crux", crux_test)]
    #[cfg_attr(not(feature="verifier-crux"), test)]
    fn bounds_should_fail(_a: u32) {
        let a: [u32; 3] = [1, 2, 3];
        assert_eq!(a[3], 4)
    }
}

proptest! {
    // #[should_panic(expected = "HELP")]
    #[cfg_attr(feature="verifier-crux", crux_test)]
    #[cfg_attr(not(feature="verifier-crux"), test)]
    fn panic_should_fail(_a: u32) {
        panic!("HELP")
    }
}

proptest! {
    // #[should_panic(expected = "assertion")]
    #[cfg_attr(feature="verifier-crux", crux_test)]
    #[cfg_attr(not(feature="verifier-crux"), test)]
    fn std_assert_should_fail(_a: u32) {
        std::assert!(false)
    }
}

proptest! {
    // #[should_panic(expected = "assertion")]
    #[cfg_attr(feature="verifier-crux", crux_test)]
    #[cfg_attr(not(feature="verifier-crux"), test)]
    fn prop_assert_should_fail(_a: u32) {
        prop_assert!(false)
    }
}

proptest! {
    // #[should_panic(expected = "assertion failed: `(left == right)`")]
    #[cfg_attr(feature="verifier-crux", crux_test)]
    #[cfg_attr(not(feature="verifier-crux"), test)]
    fn assert_eq_should_fail(_a: u32) {
        assert_eq!(1, 2)
    }
}

proptest! {
    // #[should_panic(expected = "Option::unwrap()")]
    #[cfg_attr(feature="verifier-crux", crux_test)]
    #[cfg_attr(not(feature="verifier-crux"), test)]
    fn unwrap_should_fail(_a: u32) {
        None.unwrap()
    }
}
