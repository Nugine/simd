use vsimd::mask::u8x32_highbit_any;
use vsimd::SIMD256;

use core::ops::Not;

#[inline(always)]
pub unsafe fn is_ascii_ct_fallback(mut src: *const u8, len: usize) -> bool {
    let mut ans = 0;
    let end = src.add(len);
    while src < end {
        ans |= src.read();
        src = src.add(1);
    }
    ans < 0x80
}

#[inline(always)]
pub unsafe fn is_ascii_ct_simd<S: SIMD256>(s: S, mut src: *const u8, mut len: usize) -> bool {
    let end = src.add(len / 32 * 32);
    let mut y = s.v256_create_zero();
    while src < end {
        let x = s.v256_load_unaligned(src);
        y = s.v256_or(y, x);
        src = src.add(32);
    }
    len %= 32;

    let mut ans = u8x32_highbit_any(s, y).not();
    ans &= is_ascii_ct_fallback(src, len);
    ans
}
