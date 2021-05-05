#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

use regex::Regex;

proptest! {
    #[test]
    // Construct an arbitrary (utf8) string from 3 bytes.
    // Klee can only handle a small number of bytes in this case.
    fn string(s in prop::string::arbitrary(3)) {
        let re = Regex::new(r"^a").unwrap();
        prop_assume!(re.is_match(&s));
        prop_assert!(s.starts_with('a'));
    }
}

proptest! {
    #[test]
    // Construct a (utf8) string from 100 bytes, restricted to ascii chars.
    // Klee can handle much more bytes this way.
    fn ascii_string(s in prop::string::arbitrary_ascii(100)) {
        let re = Regex::new(r"^a").unwrap();
        prop_assume!(re.is_match(&s));
        prop_assert!(s.starts_with('a'));
    }
}

proptest! {
    #[test]
    #[should_panic]
    fn string_add(s1 in prop::string::arbitrary_ascii(200), s2 in prop::string::arbitrary_ascii(200)) {
        let s = s1 + &s2;
        let s: String = s.chars().rev().collect();
        prop_assert!(s.len() == 200);
    }
}
