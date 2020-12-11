#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

#[link(name = "bar_library")]
extern {
    fn bar_function(x: i32) -> i32;
}

fn bar(x: i32) -> i32 {
    unsafe {
        bar_function(x)
    }
}

proptest!{
    fn main(i in any::<i32>()) {
        prop_assert!(bar(i) != i)
    }
}

proptest! {
    #[test]
    fn inequal(x in any::<i32>()) {
        prop_assert!(bar(x) != x)
    }
}

proptest! {
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn greater(x in any::<i32>()) {
        prop_assert!(bar(x) > x)
    }
}
