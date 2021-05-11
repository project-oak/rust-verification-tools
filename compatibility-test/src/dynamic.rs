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
// trait objects
////////////////////////////////////////////////////////////////

#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

////////////////////////////////////////////////////////////////
// A trait and a couple of implementations
////////////////////////////////////////////////////////////////

#[cfg(test)]
mod foo {
    use core::fmt::Debug;

    pub trait Foo: Debug {
        fn foo(&self) -> i32;
    }

    // A boxed trait object type
    pub type FB = Box<dyn Foo>;

    #[derive(Debug)]
    struct A {
        a: i8,
    }
    impl Foo for A {
        fn foo(&self) -> i32 {
            self.a.into()
        }
    }
    pub fn a_to_foo(a: i8) -> FB {
        Box::new(A { a })
    }

    #[derive(Debug)]
    struct B {
        b: i16,
    }
    impl Foo for B {
        fn foo(&self) -> i32 {
            self.b.into()
        }
    }
    pub fn b_to_foo(b: i16) -> FB {
        Box::new(B { b })
    }
}

////////////////////////////////////////////////////////////////
// Proptest-based tests exploring how to use proptest with
// trait objects
////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    proptest! {
        #[test]
        fn dynamic(x in (0..10i8).prop_map(foo::a_to_foo).boxed()) {
            // println!("x = {:?}", x);
            let y : i32 = x.foo();
            assert!(y != 15);
            assert!((0..10).contains(&y));
        }
    }

    proptest! {
        #[test]
        fn dynamic_union(r in (0..10i8).prop_map(|x| foo::a_to_foo(x)).boxed().prop_union(
                              (1000i16..).prop_map(foo::b_to_foo).boxed())) {
            // println!("r = {:?}", r);
            assert!(r.foo() < 10 || r.foo() > 100);
        }
    }

    proptest! {
        #[test]
        fn dynamic_oneof(r in prop_oneof![
                                 (0..10i8).prop_map(foo::a_to_foo),
                                 (1000i16..).prop_map(foo::b_to_foo)]) {
            // println!("r = {:?}", r);
            assert!(r.foo() < 10 || r.foo() > 100);
        }
    }
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
