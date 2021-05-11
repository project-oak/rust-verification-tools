// Copyright 2020 The Propverify authors
// Based on parts of Proptest which is Copyright 2017, 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

////////////////////////////////////////////////////////////////
// Proptest-based tests exploring how to use proptest with
// enumerations
////////////////////////////////////////////////////////////////

#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

////////////////////////////////////////////////////////////////
// An enumeration type
//
// Example taken from proptest
////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
    use core::fmt::Debug;

    #[derive(Clone, Debug)]
    enum MyEnum {
        Big(u64),
        Medium(u32),
        Little(i16),
    }

    fn my_enum_strategy(s: u8) -> impl Strategy<Value = MyEnum> {
        prop_oneof![
            (0..i16::from(s)).prop_map(MyEnum::Little),
            (0..u32::from(s)).prop_map(MyEnum::Medium),
            (0..u64::from(s)).prop_map(MyEnum::Big),
        ]
    }

    proptest! {
        #[test]
        fn enum_test1(x in my_enum_strategy(10)) {
            match x {
                MyEnum::Big(b) => assert!(b < 10),
                MyEnum::Medium(m) => assert!(m < 10),
                MyEnum::Little(l) => assert!(l < 10)
            }
        }
    }
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
