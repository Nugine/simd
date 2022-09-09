use vsimd::hex::unhex;
use vsimd::scalar::align32;
use vsimd::SIMD256;

use core::ops::Not;

#[inline]
pub fn check_fallback(data: &[u8]) -> bool {
    let mut iter = data.chunks_exact(4);
    for chunk in &mut iter {
        let y1 = unhex(chunk[0]);
        let y2 = unhex(chunk[1]);
        let y3 = unhex(chunk[2]);
        let y4 = unhex(chunk[3]);
        if y1 | y2 | y3 | y4 == 0xff {
            return false;
        }
    }
    let flag = iter.remainder().iter().fold(0, |acc, &x| acc | unhex(x));
    flag != 0xff
}

#[inline]
pub fn check_simd<S: SIMD256>(s: S, data: &[u8]) -> bool {
    let (prefix, middle, suffix) = align32(data);

    if check_fallback(prefix).not() {
        return false;
    }

    for &chunk in middle {
        if vsimd::hex::check_ascii32(s, chunk).not() {
            return false;
        }
    }

    check_fallback(suffix)
}
