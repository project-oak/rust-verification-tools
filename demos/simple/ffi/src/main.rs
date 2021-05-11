#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

#[link(name = "bar_library")]
extern "C" {
    fn bar_function(x: i32) -> i32;
}

fn bar(x: i32) -> i32 {
    unsafe { bar_function(x) }
}

proptest! {
    fn main(i: i32) {
        prop_assert!(bar(i) != i)
    }
}

proptest! {
    #[test]
    fn inequal(x: i32) {
        prop_assert!(bar(x) != x)
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn greater(x: i32) {
        prop_assert!(bar(x) > x)
    }
}
