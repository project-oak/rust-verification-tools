use std::arch::x86_64::*;
use std::mem::transmute;

unsafe fn test_mm_cmpeq_epi8(x: i8) {
    let a = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    let b = _mm_setr_epi8(15, 14, x, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let r = _mm_cmpeq_epi8(a, b);
    assert_eq_m128i(
        r,
        _mm_setr_epi8(0, 0, 0xFFu8 as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    );
}

unsafe fn test_mm_movemask_epi8(x: i8) {
    let a = _mm_set1_epi8(x);
    let r = _mm_movemask_epi8(a);
    println!("movemask({:?}) = {}", a, r);
}

unsafe fn test_mm_xor_si128() {
    let a = _mm_set1_epi8(5);
    let b = _mm_set1_epi8(3);
    let r = _mm_xor_si128(a, b);
    assert_eq_m128i(r, _mm_set1_epi8(6));
}

pub unsafe fn assert_eq_m128i(a: __m128i, b: __m128i) {
    assert_eq!(transmute::<_, [u64; 2]>(a), transmute::<_, [u64; 2]>(b))
}

fn main() {
    println!("Hello, world!");
    unsafe { test_mm_cmpeq_epi8(2) };
    unsafe { test_mm_cmpeq_epi8(2) };
    unsafe { test_mm_xor_si128() };
    for i in -3..3 {
        unsafe { test_mm_movemask_epi8(i) };
    }
}
