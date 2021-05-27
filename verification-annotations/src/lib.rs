// Copyright 2020 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(cstring_from_vec_with_nul)]
#![cfg_attr(not(feature = "std"), no_std)]

// Traits for creating symbolic/abstract values
pub mod traits;
pub mod verifier;

#[cfg(feature = "std")]
pub mod utils {
    pub trait UnwrapOrReject {
        type Wrapped;
        fn unwrap_or_reject(self) -> Self::Wrapped;
    }

    impl<T, E> UnwrapOrReject for Result<T, E> {
        type Wrapped = T;
        fn unwrap_or_reject(self) -> Self::Wrapped {
            match self {
                Ok(x) => x,
                Err(_) => crate::verifier::reject(),
            }
        }
    }

    impl<T> UnwrapOrReject for Option<T> {
        type Wrapped = T;
        fn unwrap_or_reject(self) -> Self::Wrapped {
            match self {
                Some(x) => x,
                None => crate::verifier::reject(),
            }
        }
    }
}

// `use verfication_annotations::prelude::*`
pub mod prelude {
    pub use crate::traits::*;
    #[cfg(feature = "std")]
    pub use crate::utils::*;
    pub use crate::verifier;

    // Macros
    pub use crate::verifier::assert as verifier_assert;
    pub use crate::verifier::assert_eq as verifier_assert_eq;
    pub use crate::verifier::assert_ne as verifier_assert_ne;
    pub use crate::verifier::assume as verifier_assume;
    pub use crate::verifier::unreachable as verifier_unreachable;
}

// At the moment, the cargo-verify script does not support
// use of a separate test directory so, for now, we put
// the tests here.
#[cfg(all(test, feature = "std"))]
mod tests;
