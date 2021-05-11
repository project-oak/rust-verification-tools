// Copyright 2020 The Propverify authors
// Based on parts of Proptest which is Copyright 2017, 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod strategy;

pub mod prelude {
    // Macros
    pub use crate::prop_assume;
    pub use crate::prop_compose;
    pub use crate::prop_oneof;
    pub use crate::proptest;

    // Functions and types
    pub use crate::strategy::of;
    pub use crate::strategy::prop_is_replay;
    pub use crate::strategy::Just;
    pub use crate::strategy::Strategy;
    pub use crate::strategy::{maybe_err, maybe_ok};

    // Modules with same name as types
    pub use crate::strategy::{bool, char};

    // Arbitrary trait
    pub use crate::strategy::{any, Arbitrary};

    pub mod prop {
        pub use crate::strategy::prop_is_replay;
        pub use crate::strategy::{uniform0, uniform1, uniform2, uniform3, uniform4};
        pub use crate::strategy::{uniform10, uniform11, uniform12, uniform13, uniform14};
        pub use crate::strategy::{uniform15, uniform16, uniform17, uniform18, uniform19};
        pub use crate::strategy::{uniform20, uniform21, uniform22, uniform23, uniform24};
        pub use crate::strategy::{uniform25, uniform26, uniform27, uniform28, uniform29};
        pub use crate::strategy::{uniform30, uniform31, uniform32};
        pub use crate::strategy::{uniform5, uniform6, uniform7, uniform8, uniform9};
        pub mod collection {
            pub use crate::strategy::binary_heap;
            pub use crate::strategy::btree_map;
            pub use crate::strategy::btree_set;
            pub use crate::strategy::linked_list;
            pub use crate::strategy::vec;
            pub use crate::strategy::vec_deque;
        }
        pub mod num {
            #[cfg(feature = "float")]
            pub use crate::strategy::{f32, f64};
            pub use crate::strategy::{i128, i16, i32, i64, i8, isize};
            pub use crate::strategy::{u128, u16, u32, u64, u8, usize};
        }

        pub use crate::strategy::string;
    }

    pub use verification_annotations;
    pub use verification_annotations::verifier;

    pub use verifier::assert as prop_assert;
    pub use verifier::assert_eq as prop_assert_eq;
    pub use verifier::assert_ne as prop_assert_ne;
}
