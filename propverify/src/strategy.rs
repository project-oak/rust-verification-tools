// Copyright 2020 The Propverify authors
// Based on parts of Proptest which is Copyright 2017, 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use verification_annotations::prelude::*;

use std::boxed::Box;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

use std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
// use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

// Trait representing a set of values from which one can be chosen
//
// The primary method is `value` chooses a value from the set.
//
// The other methods are copied from the proptest Strategy trait - see the documentation
// for proptest.
//
// Implementations of this trait are datatypes such as Any, Just, VecStrategy, etc.
// and, in some cases, these datatypes mirror the type structure for which they
// generate values.
//
// Strategies for composite types (tuples, vectors, etc.) typically contain
// strategies for generating components of that type (e.g., the struct fields,
// array/vector elements, etc.)
pub trait Strategy: std::fmt::Debug {
    type Value: std::fmt::Debug;
    fn value(&self) -> Self::Value;

    fn prop_map<O, F: Fn(Self::Value) -> O>(self, fun: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map {
            source: self,
            fun: Arc::new(fun),
        }
    }

    fn prop_map_into<O>(self) -> MapInto<Self, O>
    where
        Self: Sized,
        Self::Value: Into<O>,
    {
        MapInto {
            source: self,
            output: PhantomData,
        }
    }

    fn prop_flat_map<S: Strategy, F: Fn(Self::Value) -> S>(self, fun: F) -> Flatten<Map<Self, F>>
    where
        Self: Sized,
    {
        Flatten {
            source: Map {
                source: self,
                fun: Arc::new(fun),
            },
        }
    }

    // Todo: In proptest, the only difference between prop_flat_map
    // and prop_ind_flat_map is in how they shrink.
    // So it is not clear that there is any point in having
    // this method. Or, maybe this method should exist for compatibility
    // but it should just call prop_flat_map
    fn prop_ind_flat_map<S: Strategy, F: Fn(Self::Value) -> S>(
        self,
        fun: F,
    ) -> IndFlatten<Map<Self, F>>
    where
        Self: Sized,
    {
        IndFlatten {
            source: Map {
                source: self,
                fun: Arc::new(fun),
            },
        }
    }

    // Todo: In proptest, the only difference between prop_flat_map
    // and prop_ind_flat_map2 is in how they shrink and that
    // prop_ind_flat_map2 returns a tuple of type `(Self::Value, S)`.
    // Maybe, it is not needed or, for compatibility with proptest,
    // it should be implemented with a call to prop_flat_map.
    fn prop_ind_flat_map2<S: Strategy, F: Fn(Self::Value) -> S>(
        self,
        fun: F,
    ) -> IndFlattenMap<Self, F>
    where
        Self: Sized,
    {
        IndFlattenMap {
            source: self,
            fun: Arc::new(fun),
        }
    }

    fn prop_filter<F: Fn(&Self::Value) -> bool>(self, _whence: &str, fun: F) -> Filter<Self, F>
    where
        Self: Sized,
    {
        Filter {
            source: self,
            fun: Arc::new(fun),
        }
    }

    fn prop_filter_map<F: Fn(Self::Value) -> Option<O>, O>(
        self,
        _whence: &str,
        fun: F,
    ) -> FilterMap<Self, F>
    where
        Self: Sized,
    {
        FilterMap {
            source: self,
            fun: Arc::new(fun),
        }
    }

    fn prop_union(self, other: Self) -> Union<Self>
    where
        Self: Sized,
    {
        Union { x: self, y: other }
    }

    fn boxed(self) -> BoxedStrategy<Self::Value>
    where
        Self: Sized + 'static,
    {
        BoxedStrategy { b: Box::new(self) }
    }
}

pub trait Arbitrary: Sized + std::fmt::Debug {
    type Strategy: Strategy<Value = Self>;
    fn arbitrary() -> Self::Strategy;
}

pub type StrategyFor<A> = <A as Arbitrary>::Strategy;

pub fn any<A: Arbitrary>() -> StrategyFor<A> {
    // ^-- We use a shorter name so that turbofish becomes more ergonomic.
    A::arbitrary()
}

// It appears that if a macro refers to an import that has been renamed
// using 'use X as Y;', then the macro cannot refer to 'Y::foo'
// but it can refer to functions defined in the same crate as the macro.
// So this is a small stub redirects to verifier::is_replay.
pub fn prop_is_replay() -> bool {
    verifier::is_replay()
}

#[macro_export]
macro_rules! proptest {
    (
      $(#[$meta:meta])*
      fn $test_name:ident($($parm:tt in $strategy:expr),+ $(,)?) $body:block
    ) => {
      #[cfg_attr(crux, crux_test)]
      $(#[cfg_attr(not(crux), $meta)])*
      fn $test_name() {
          $(
              #[cfg(not(crux))]
              {
                  let str = stringify!($meta);
                  if str.starts_with("should_panic") {
                      verifier::expect_raw(str);
                  }
              }
          )*
          $(let $parm = $crate::prelude::Strategy::value(&$strategy);)*

          #[cfg(not(crux))]
          if prop_is_replay() {
              $(println!("  Value {} = {:?}", std::stringify!($parm), $parm);)*
          }

          $body
      }
    };
    (
      $(#[$meta:meta])*
      fn $test_name:ident($($parm:ident : $s:ty),+ $(,)?) $body:block
    ) => {
        $crate::proptest!{
            $(#[$meta])*
            fn $test_name($($parm in $crate::prelude::any::<$s>()),+) $body
        }
    };
}

/// Assume that condition `cond` is true
#[macro_export]
macro_rules! prop_assume {
    ($expr:expr) => {
        verifier::assume($expr)
    };

    ($expr:expr, $fmt:tt $(, $fmt_arg:expr),*) => {
        verifier::assume($expr)
    };
}

// Combine multiple strategies into a single strategy
#[macro_export]
macro_rules! prop_oneof {
    ($item:expr $(,)?) => {
        std::sync::Arc::new($item).boxed()
    };
    ($first:expr, $($rest:expr),* $(,)?) => {
        $crate::prelude::Strategy::prop_union(std::sync::Arc::new($first).boxed(), $crate::prop_oneof![$($rest),*]).boxed()
    };
}

// Defining complex strategies
#[macro_export]
macro_rules! prop_compose {
    ($(#[$meta:meta])*
     $vis:vis
     $([$($modi:tt)*])? fn $name:ident $params:tt
     ($($var:pat in $strategy:expr),+ $(,)?)
       -> $return_type:ty $body:block
    ) => {
        #[must_use = "strategies do nothing unless used"]
        $(#[$meta])*
        $vis
        $($($modi)*)? fn $name $params
                 -> impl $crate::prelude::Strategy<Value = $return_type> {
            let strat = $crate::proptest_helper!(@_STRATS2TUPLE ($($strategy)*));
            $crate::prelude::Strategy::prop_map(strat,
                move |$crate::proptest_helper!(@_PATS2TUPLEPAT ($($var),*))| $body)
        }
    };
}

// Support macro to help write prop_compose
// (Somewhat reduced from the proptest version of this macro
// to make it easier for me to grasp what it does.)
//
// This macro seems to be several macros combined into one with the
// outermost @_<tag> selecting which behaivour is actually wanted.
// I don't understand the motivation behind this yet.
#[doc(hidden)]
#[macro_export]
macro_rules! proptest_helper {
    // First set of conversions take a list of strategies and convert them to a tuple
    (@_STRATS2TUPLE ($a:tt)) => {
        $a
    };
    (@_STRATS2TUPLE ($a0:tt $a1:tt)) => {
        ($a0, $a1)
    };
    (@_STRATS2TUPLE ($a0:tt $a1:tt $a2:tt)) => {
        ($a0, $a1, $a2)
    };
    (@_STRATS2TUPLE ($a0:tt $a1:tt $a2:tt $a3:tt)) => {
        ($a0, $a1, $a2, $a3)
    };

    // Second set of conversions take a list of patterns and convert them to a tuple of patterns
    // (The patterns might be simple variables?)
    (@_PATS2TUPLEPAT ($item:pat)) => {
        $item
    };
    (@_PATS2TUPLEPAT ($a0:pat, $a1:pat)) => {
        ($a0, $a1)
    };
    (@_PATS2TUPLEPAT ($a0:pat, $a1:pat, $a2:pat)) => {
        ($a0, $a1, $a2)
    };
    (@_PATS2TUPLEPAT ($a0:pat, $a1:pat, $a2:pat, $a3:pat)) => {
        ($a0, $a1, $a2, $a3)
    };
}

// The remainder of this file consists of implementations of the Strategy trait.
// In most cases, this consists of defining a new struct type to represent
// the strategy, defining functions to construct that struct type and
// then implementing the Strategy trait for that type.

// The most trivial strategy
#[derive(Clone, Copy, Debug)]
pub struct Just<T: Clone>(pub T);
impl<T: Clone + std::fmt::Debug> Strategy for Just<T> {
    type Value = T;
    fn value(&self) -> Self::Value {
        self.0.clone()
    }
}

impl<T: std::fmt::Debug> Strategy for fn() -> T {
    type Value = T;

    fn value(&self) -> Self::Value {
        self()
    }
}

pub mod bool {
    use super::*;
    #[derive(Clone, Copy, Debug)]
    pub struct Any(());
    pub const ANY: Any = Any(());
    impl Strategy for Any {
        type Value = bool;
        fn value(&self) -> Self::Value {
            let c: u8 = verifier::AbstractValue::abstract_value();
            verifier::assume(c == 0 || c == 1);
            c == 1
        }
    }
    impl Arbitrary for bool {
        type Strategy = Any;
        fn arbitrary() -> Self::Strategy {
            ANY
        }
    }
}

pub mod char {
    use super::*;
    #[derive(Clone, Copy, Debug)]
    pub struct Any(());
    pub const ANY: Any = Any(());
    impl Strategy for Any {
        type Value = char;
        fn value(&self) -> Self::Value {
            let c: u32 = verifier::AbstractValue::abstract_value();
            std::char::from_u32(c).unwrap_or_reject()
        }
    }
    impl Arbitrary for char {
        type Strategy = Any;
        fn arbitrary() -> Self::Strategy {
            ANY
        }
    }
}

#[derive(Clone)]
pub struct Map<S, F> {
    source: S,
    fun: Arc<F>,
}

impl<S: std::fmt::Debug, F> std::fmt::Debug for Map<S, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Map")
            .field("source", &self.source)
            .field("fun", &"<function>")
            .finish()
    }
}

impl<S: Strategy, T: std::fmt::Debug, F: Fn(S::Value) -> T> Strategy for Map<S, F> {
    type Value = T;
    fn value(&self) -> Self::Value {
        let val = self.source.value();
        (self.fun)(val)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MapInto<S: Strategy, T> {
    source: S,
    output: PhantomData<T>,
}
impl<S: Strategy, T: std::fmt::Debug> Strategy for MapInto<S, T>
where
    S::Value: Into<T>,
{
    type Value = T;
    fn value(&self) -> Self::Value {
        let val = self.source.value();
        val.into()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct IndFlatten<S> {
    source: S,
}
impl<S: Strategy> Strategy for IndFlatten<S>
where
    S::Value: Strategy,
{
    type Value = <S::Value as Strategy>::Value;
    fn value(&self) -> Self::Value {
        self.source.value().value()
    }
}

#[derive(Clone)]
pub struct IndFlattenMap<S, F> {
    source: S,
    fun: Arc<F>,
}

impl<S: std::fmt::Debug, F> std::fmt::Debug for IndFlattenMap<S, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("IndFlattenMap")
            .field("source", &self.source)
            .field("fun", &"<function>")
            .finish()
    }
}

impl<S: Strategy, T: Strategy, F: Fn(S::Value) -> T> Strategy for IndFlattenMap<S, F>
where
    S::Value: Copy,
{
    type Value = (S::Value, T::Value);
    fn value(&self) -> Self::Value {
        let s = self.source.value();
        let r = (self.fun)(s).value();
        (s, r)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Flatten<S> {
    source: S,
}
impl<S: Strategy> Strategy for Flatten<S>
where
    S::Value: Strategy,
{
    type Value = <S::Value as Strategy>::Value;
    fn value(&self) -> Self::Value {
        let val = self.source.value();
        val.value()
    }
}

#[derive(Clone)]
pub struct Filter<S, F> {
    source: S,
    fun: Arc<F>,
}

impl<S: std::fmt::Debug, F> std::fmt::Debug for Filter<S, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Filter")
            .field("source", &self.source)
            .field("fun", &"<function>")
            .finish()
    }
}

impl<S: Strategy, F: Fn(&S::Value) -> bool> Strategy for Filter<S, F> {
    type Value = S::Value;
    fn value(&self) -> Self::Value {
        let val = self.source.value();
        verifier::assume((self.fun)(&val));
        val
    }
}

#[derive(Clone)]
pub struct FilterMap<S, F> {
    source: S,
    fun: Arc<F>,
}

impl<S: std::fmt::Debug, F> std::fmt::Debug for FilterMap<S, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FilterMap")
            .field("source", &self.source)
            .field("fun", &"<function>")
            .finish()
    }
}

impl<S: Strategy, F: Fn(S::Value) -> Option<T>, T: std::fmt::Debug> Strategy for FilterMap<S, F> {
    type Value = T;
    fn value(&self) -> Self::Value {
        let val = self.source.value();
        (self.fun)(val).unwrap_or_reject()
    }
}

#[derive(Clone, Debug)]
pub struct Union<S> {
    x: S,
    y: S,
}
impl<S: Strategy> Strategy for Union<S> {
    type Value = S::Value;
    fn value(&self) -> Self::Value {
        if verifier::AbstractValue::abstract_value() {
            self.x.value()
        } else {
            self.y.value()
        }
    }
}

macro_rules! proxy_strategy {
    ($typ:ty $(, $lt:tt)*) => {
        impl<$($lt,)* S : Strategy + ?Sized> Strategy for $typ {
            type Value = S::Value;

            fn value(&self) -> Self::Value {
                (**self).value()
            }
        }
    };
}
proxy_strategy!(Box<S>);
proxy_strategy!(&'a S, 'a);
proxy_strategy!(&'a mut S, 'a);
proxy_strategy!(Rc<S>);
proxy_strategy!(Arc<S>);

#[derive(Debug)]
pub struct BoxedStrategy<T> {
    b: Box<dyn Strategy<Value = T>>,
}
impl<T: std::fmt::Debug> Strategy for BoxedStrategy<T> {
    type Value = T;
    fn value(&self) -> Self::Value {
        self.b.value()
    }
}

macro_rules! numeric_api {
    ( $( $typ:ident; )* ) => {
        $(
            pub mod $typ {
                use super::*;
                #[derive(Clone, Copy, Debug)]
                pub struct Any(());
                pub const ANY: Any = Any(());
                impl Strategy for Any {
                    type Value = $typ;
                    fn value(&self) -> Self::Value {
                        let r : $typ = verifier::AbstractValue::abstract_value();
                        r
                    }
                }
                impl Arbitrary for $typ {
                    type Strategy = Any;
                    fn arbitrary() -> Self::Strategy { ANY }
                }
            }

            impl Strategy for ::core::ops::Range<$typ> {
                type Value = $typ;
                fn value(&self) -> Self::Value {
                    let r : $typ = verifier::AbstractValue::abstract_value();
                    verifier::assume(self.start <= r);
                    verifier::assume(r < self.end);
                    r
                }
            }

            impl Strategy for ::core::ops::RangeInclusive<$typ> {
                type Value = $typ;
                fn value(&self) -> Self::Value {
                    let r : $typ = verifier::AbstractValue::abstract_value();
                    verifier::assume(*self.start() <= r);
                    verifier::assume(r <= *self.end());
                    r
                }
            }

            impl Strategy for ::core::ops::RangeFrom<$typ> {
                type Value = $typ;
                fn value(&self) -> Self::Value {
                    let r : $typ = verifier::AbstractValue::abstract_value();
                    verifier::assume(self.start <= r);
                    r
                }
            }

            impl Strategy for ::core::ops::RangeTo<$typ> {
                type Value = $typ;
                fn value(&self) -> Self::Value {
                    let r : $typ = verifier::AbstractValue::abstract_value();
                    verifier::assume(r < self.end);
                    r
                }
            }

            impl Strategy for ::core::ops::RangeToInclusive<$typ> {
                type Value = $typ;
                fn value(&self) -> Self::Value {
                    let r : $typ = verifier::AbstractValue::abstract_value();
                    verifier::assume(r <= self.end);
                    r
                }
            }

        )*
    }
}

numeric_api! {
    u8;
    u16;
    u32;
    u64;
    u128;
    usize;
    i8;
    i16;
    i32;
    i64;
    i128;
    isize;
}

#[cfg(feature = "float")]
numeric_api! {
    f32;
    f64;
}

macro_rules! strategic_tuple {
    {$($idx:tt => $s:ident;)*} => {

        impl<$($s: Strategy),*> Strategy for ($($s),*) {
            type Value = ($($s::Value,)*);
            fn value(&self) -> Self::Value {
                ($(self.$idx.value()),*)
            }
        }
    };
}
// todo: It should be possible to write a macro that generates this sequence
strategic_tuple! {}
// A Tuple1 instance would create a warning
strategic_tuple! {0=>A; 1=>B;}
strategic_tuple! {0=>A; 1=>B; 2=>C;}
strategic_tuple! {0=>A; 1=>B; 2=>C; 3=>D;}
strategic_tuple! {0=>A; 1=>B; 2=>C; 3=>D; 4=>E;}
strategic_tuple! {0=>A; 1=>B; 2=>C; 3=>D; 4=>E; 5=>F;}
strategic_tuple! {0=>A; 1=>B; 2=>C; 3=>D; 4=>E; 5=>F; 6=>G;}
strategic_tuple! {0=>A; 1=>B; 2=>C; 3=>D; 4=>E; 5=>F; 6=>G; 7=>H;}
strategic_tuple! {0=>A; 1=>B; 2=>C; 3=>D; 4=>E; 5=>F; 6=>G; 7=>H; 8=>I;}
strategic_tuple! {0=>A; 1=>B; 2=>C; 3=>D; 4=>E; 5=>F; 6=>G; 7=>H; 8=>I; 9=>J;}
strategic_tuple! {0=>A; 1=>B; 2=>C; 3=>D; 4=>E; 5=>F; 6=>G; 7=>H; 8=>I; 9=>J; 10=>K;}
strategic_tuple! {0=>A; 1=>B; 2=>C; 3=>D; 4=>E; 5=>F; 6=>G; 7=>H; 8=>I; 9=>J; 10=>K; 11=>L;}

// Array strategy where S is element strategy and T is [S::Value; n] for some n
#[derive(Clone, Copy, Debug)]
pub struct ArrayStrategy<S, T> {
    s: S,
    _marker: PhantomData<T>,
}
impl<S: Strategy, T> ArrayStrategy<S, T> {
    pub fn new(s: S) -> Self {
        Self {
            s,
            _marker: PhantomData,
        }
    }
}

macro_rules! small_array {
    ($n:tt $name:ident : $($elt:ident),*) => {
        pub fn $name<S: Strategy> (s: S) -> ArrayStrategy<S, [S::Value; $n]>
        {
            ArrayStrategy {
                s,
                _marker: PhantomData,
            }
        }

        impl<S: Strategy> Strategy for ArrayStrategy<S, [S::Value; $n]>
        {
            type Value = [S::Value; $n];
            fn value(&self) -> Self::Value {
                $(let $elt = self.s.value();)*
                [$($elt),*]
            }
        }
    }
}

// todo: it should be possible to write a macro that generates this sequence
small_array!(0  uniform0 : );
small_array!(1  uniform1 : a0);
small_array!(2  uniform2 : a0, a1);
small_array!(3  uniform3 : a0, a1, a2);
small_array!(4  uniform4 : a0, a1, a2, a3);
small_array!(5  uniform5 : a0, a1, a2, a3, a4);
small_array!(6  uniform6 : a0, a1, a2, a3, a4, a5);
small_array!(7  uniform7 : a0, a1, a2, a3, a4, a5, a6);
small_array!(8  uniform8 : a0, a1, a2, a3, a4, a5, a6, a7);
small_array!(9  uniform9 : a0, a1, a2, a3, a4, a5, a6, a7, a8);
small_array!(10 uniform10: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
small_array!(11 uniform11: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10);
small_array!(12 uniform12: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11);
small_array!(13 uniform13: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12);
small_array!(14 uniform14: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13);
small_array!(15 uniform15: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14);
small_array!(16 uniform16: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15);
small_array!(17 uniform17: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16);
small_array!(18 uniform18: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17);
small_array!(19 uniform19: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18);
small_array!(20 uniform20: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19);
small_array!(21 uniform21: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20);
small_array!(22 uniform22: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21);
small_array!(23 uniform23: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22);
small_array!(24 uniform24: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22, a23);
small_array!(25 uniform25: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22, a23, a24);
small_array!(26 uniform26: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22, a23, a24, a25);
small_array!(27 uniform27: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22, a23, a24, a25, a26);
small_array!(28 uniform28: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22, a23, a24, a25, a26, a27);
small_array!(29 uniform29: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22, a23, a24, a25, a26, a27, a28);
small_array!(30 uniform30: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22, a23, a24, a25, a26, a27, a28, a29);
small_array!(31 uniform31: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22, a23, a24, a25, a26, a27, a28, a29,
                           a30);
small_array!(32 uniform32: a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
                           a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
                           a20, a21, a22, a23, a24, a25, a26, a27, a28, a29,
                           a30, a31);

#[derive(Clone, Copy, Debug)]
pub struct OptionStrategy<S> {
    s: S,
}
impl<S: Strategy> Strategy for OptionStrategy<S>
where
    S: Strategy + Clone,
{
    type Value = Option<S::Value>;
    fn value(&self) -> Self::Value {
        if bool::ANY.value() {
            Some(self.s.value())
        } else {
            None
        }
    }
}

pub fn of<S: Strategy>(s: S) -> OptionStrategy<S> {
    OptionStrategy { s }
}

#[derive(Clone, Copy, Debug)]
pub struct ResultStrategy<A, B> {
    a: A,
    b: B,
}
impl<A, B> Strategy for ResultStrategy<A, B>
where
    A: Strategy + Clone,
    B: Strategy + Clone,
{
    type Value = Result<A::Value, B::Value>;
    fn value(&self) -> Self::Value {
        if bool::ANY.value() {
            Ok(self.a.value())
        } else {
            Err(self.b.value())
        }
    }
}

pub fn maybe_ok<A: Strategy, B: Strategy>(a: A, b: B) -> ResultStrategy<A, B> {
    ResultStrategy { a, b }
}

pub fn maybe_err<A: Strategy, B: Strategy>(a: A, b: B) -> ResultStrategy<A, B> {
    ResultStrategy { a, b }
}

#[derive(Clone, Copy, Debug)]
pub struct VecStrategy<S: Strategy> {
    element: S,
    size: usize, // concrete size to be more friendly to concolic/DSE
}
impl<S: Strategy> Strategy for VecStrategy<S> {
    type Value = Vec<S::Value>;
    fn value(&self) -> Self::Value {
        // Note that choosing a small, symbolic size causes KLEE to complain so
        // the length must be concrete.
        // let len = Strategy::value(&(..=self.size));
        let len = self.size;
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(self.element.value());
        }
        v
    }
}

pub fn vec<S: Strategy>(element: S, size: usize) -> VecStrategy<S> {
    VecStrategy { element, size }
}

pub mod string {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub struct Any(usize);
    // pub const ANY: Any = Any();
    impl Strategy for Any {
        type Value = String;
        fn value(&self) -> Self::Value {
            let length = self.0;
            let bytes = verifier::verifier_nondet_bytes(length);
            String::from_utf8(bytes).unwrap_or_reject()
        }
    }
    // impl Arbitrary for String {
    //     type Strategy = Any;
    //     fn arbitrary() -> Self::Strategy { ANY }
    // }
    pub fn arbitrary(length: usize) -> Any {
        Any(length)
    }

    #[derive(Clone, Copy, Debug)]
    pub struct AnyAscii(usize);
    impl Strategy for AnyAscii {
        type Value = String;
        fn value(&self) -> Self::Value {
            let length = self.0;
            let bytes = verifier::verifier_nondet_bytes(length);
            for i in 0..length {
                verifier::assume(bytes[i] != 0u8);
                verifier::assume(bytes[i].is_ascii());
            }
            String::from_utf8(bytes).unwrap_or_reject()
        }
    }
    pub fn arbitrary_ascii(length: usize) -> AnyAscii {
        AnyAscii(length)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VecDequeStrategy<S: Strategy> {
    element: S,
    size: usize, // concrete size to be more friendly to concolic/DSE
}
impl<S: Strategy> Strategy for VecDequeStrategy<S>
where
    S: Strategy + Clone,
{
    type Value = VecDeque<S::Value>;
    fn value(&self) -> Self::Value {
        // Note that choosing a small, symbolic size causes KLEE to complain so
        // the length must be concrete.
        // let len = Strategy::value(&(..=self.size));
        let len = self.size;
        let mut v = VecDeque::with_capacity(len);
        for _ in 0..len {
            v.push_front(self.element.value());
        }
        v
    }
}

pub fn vec_deque<S: Strategy>(element: S, size: usize) -> VecDequeStrategy<S> {
    VecDequeStrategy { element, size }
}

#[derive(Clone, Copy, Debug)]
pub struct LinkedListStrategy<S: Strategy> {
    element: S,
    size: usize, // concrete size to be more friendly to concolic/DSE
}
impl<S: Strategy> Strategy for LinkedListStrategy<S>
where
    S: Strategy + Clone,
{
    type Value = LinkedList<S::Value>;
    fn value(&self) -> Self::Value {
        let len = self.size;
        let mut v = LinkedList::new();
        for _ in 0..len {
            v.push_front(self.element.value());
        }
        v
    }
}

pub fn linked_list<S: Strategy>(element: S, size: usize) -> LinkedListStrategy<S> {
    LinkedListStrategy { element, size }
}

#[derive(Clone, Copy, Debug)]
pub struct BTreeMapStrategy<K: Strategy, V: Strategy> {
    keys: K,
    value: V,
    size: usize, // concrete size to be more friendly to concolic/DSE
}
impl<K: Strategy, V: Strategy> Strategy for BTreeMapStrategy<K, V>
where
    K::Value: Ord + Copy,
{
    type Value = BTreeMap<K::Value, V::Value>;
    fn value(&self) -> Self::Value {
        // Having a range of sizes up to some limit is acceptable
        // but I think it adds some overhead with little gain.
        // let len = Strategy::value(&(..=self.size));
        let len = self.size;
        let mut r = BTreeMap::new();

        // Keys are generated in increasing order to
        // reduce the number of effectively equivalent
        // paths through the generation code.
        let mut k = self.keys.value();
        for _ in 0..len {
            r.insert(k, self.value.value());
            let next = self.keys.value();
            verifier::assume(k <= next); // generate entries in fixed order
            k = next;
        }
        r
    }
}

pub fn btree_map<K: Strategy, V: Strategy>(keys: K, value: V, size: usize) -> BTreeMapStrategy<K, V>
where
    K::Value: Ord,
{
    BTreeMapStrategy { size, keys, value }
}

#[derive(Clone, Copy, Debug)]
pub struct BTreeSetStrategy<S: Strategy> {
    element: S,
    size: usize, // concrete size to be more friendly to concolic/DSE
}
impl<S: Strategy> Strategy for BTreeSetStrategy<S>
where
    S::Value: Ord + Copy,
{
    type Value = BTreeSet<S::Value>;
    fn value(&self) -> Self::Value {
        // Having a range of sizes up to some limit is acceptable
        // but I think it adds some overhead with little gain.
        // let len = Strategy::value(&(..=self.size));
        let len = self.size;
        let mut r = BTreeSet::new();

        // Keys are generated in increasing order to
        // reduce the number of effectively equivalent
        // paths through the generation code.
        let mut k = self.element.value();
        for _ in 0..len {
            r.insert(k);
            let next = self.element.value();
            verifier::assume(k <= next); // generate entries in fixed order
            k = next;
        }
        r
    }
}

pub fn btree_set<S: Strategy>(element: S, size: usize) -> BTreeSetStrategy<S>
where
    S::Value: Ord,
{
    BTreeSetStrategy { element, size }
}

#[derive(Clone, Copy, Debug)]
pub struct BinaryHeapStrategy<S: Strategy> {
    element: S,
    size: usize, // concrete size to be more friendly to concolic/DSE
}
impl<S: Strategy> Strategy for BinaryHeapStrategy<S>
where
    S::Value: Ord + Copy,
{
    type Value = BinaryHeap<S::Value>;
    fn value(&self) -> Self::Value {
        // Having a range of sizes up to some limit is acceptable
        // but I think it adds some overhead with little gain.
        // let len = Strategy::value(&(..=self.size));
        let len = self.size;
        let mut r = BinaryHeap::with_capacity(len);

        // Keys are generated in increasing order to
        // reduce the number of effectively equivalent
        // paths through the generation code.
        // (This would not be a good idea if we were checking BinaryHeap
        // but our goal is to checking code that uses BinaryHeap.)
        let mut k = self.element.value();
        for _ in 0..len {
            r.push(k);
            let next = self.element.value();
            verifier::assume(k <= next); // generate entries in fixed order
            k = next;
        }
        r
    }
}

pub fn binary_heap<S: Strategy>(element: S, size: usize) -> BinaryHeapStrategy<S>
where
    S::Value: Ord,
{
    BinaryHeapStrategy { element, size }
}
