#[cfg(test)]

mod ranges {

    proptest! {
        #[test]
        #[should_panic]
        fn overflow1(
            a in 0 .. std::i32::MAX,
            b in 0 .. std::i32::MAX,
        ) {
            assert_eq!(a + b, b + a);
        }
    }
}
