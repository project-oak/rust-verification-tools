// Based on fig.1 from: James Bornholt and Emina Torlak. 2018. Finding code that
// explodes under symbolic evaluation. Proc. ACM Program. Lang. 2, OOPSLA,
// Article 149 (November 2018), 26 pages. DOI:https://doi.org/10.1145/3276519

#![feature(unchecked_math)]

#[cfg(not(verify))]
use proptest::prelude::*;
#[cfg(verify)]
use propverify::prelude::*;

// The tests below check that the sum of any n ≤ N even integers is also even.

fn is_even(x: i32) -> bool {
    x % 2 == 0
}

const N: usize = 5;

// proptest! {
//     #[test]
//     fn t0(xs in prop::collection::vec(0i32..1000, N), n in 0usize..N+1) {
//         verifier::assert!(false);
//     }
// }

proptest! {
    #[test]
    fn t1(xs in prop::collection::vec(0i32..1000, N), n in 0usize..N+1) {
        let ys = xs.into_iter().filter(|x| is_even(*x));
        let zs = ys.take(n);
        prop_assert!(is_even(zs.sum::<i32>()));
    }
}

proptest! {
    #[test]
    fn t2(xs in prop::collection::vec(0i32..1000, N), n in 0usize..N+1) {
        let zs = xs.into_iter().take(n);
        prop_assume!(zs.clone().all(is_even));
        prop_assert!(is_even(zs.sum::<i32>()));
    }
}

proptest! {
    #[test]
    fn t3(xs in prop::collection::vec(0i32..1000, N), n in 0usize..N+1) {
        let zs: Vec<_> = xs.into_iter().take(n).collect();
        prop_assume!(zs.iter().all(|x| is_even(*x)));
        prop_assert!(is_even(zs.iter().sum::<i32>()));
    }
}
