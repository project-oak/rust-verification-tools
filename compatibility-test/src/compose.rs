// Copyright 2020 The Propverify authors
// Based on parts of Proptest which is Copyright 2017, 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

////////////////////////////////////////////////////////////////
// Proptest-based tests exploring how to use prop_compose
////////////////////////////////////////////////////////////////

#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

#[cfg(test)]
mod test {
    use super::*;
    use core::fmt::Debug;

    #[derive(Clone, Debug)]
    struct MyStruct {
        x: u32,
        y: u32,
    }

    // First version - written by hand
    fn my_struct_strategy1(max_integer: u32) -> impl Strategy<Value = MyStruct> {
        let strat = (0..max_integer, 0..max_integer);
        Strategy::prop_map(strat, move |(x, y)| MyStruct { x, y })
    }

    proptest! {
        #[test]
        fn struct_test1(s in my_struct_strategy1(10)) {
            assert!(s.x < 10);
            assert!(s.y < 10);
        }
    }

    // identical to my_struct_strategy1 but written using prop_compose!
    prop_compose! {
        fn my_struct_strategy2(max_integer: u32)
                              (x in 0..max_integer, y in 0..max_integer)
                             -> MyStruct {
             MyStruct { x, y, }
        }
    }

    proptest! {
        #[test]
        fn struct_test2(s in my_struct_strategy2(10)) {
            assert!(s.x < 10);
            assert!(s.y < 10);
        }
    }
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
