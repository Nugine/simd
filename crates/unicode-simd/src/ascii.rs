use vsimd::mask::u8x32_highbit_any;
use vsimd::tools::{slice, slice_parts};
use vsimd::SIMD256;

use core::ops::Not;

#[inline(always)]
pub fn is_ascii_ct_fallback(data: &[u8]) -> bool {
    unsafe {
        let mut ans = 0;
        let (mut src, len) = slice_parts(data);
        let end = src.add(len);
        while src < end {
            ans |= src.read();
            src = src.add(1);
        }
        ans < 0x80
    }
}

#[inline(always)]
pub fn is_ascii_ct_simd<S: SIMD256>(s: S, data: &[u8]) -> bool {
    unsafe {
        let (mut src, mut len) = slice_parts(data);

        let end = src.add(len / 32 * 32);
        let mut y = s.v256_create_zero();
        while src < end {
            let x = s.v256_load_unaligned(src);
            y = s.v256_or(y, x);
            src = src.add(32);
        }
        len %= 32;

        let mut ans = u8x32_highbit_any(s, y).not();
        ans &= is_ascii_ct_fallback(slice(src, len));
        ans
    }
}
