use crate::fallback::utf32 as fallback;

use vsimd::tools::{slice, slice_parts};
use vsimd::SIMD256;

#[inline(always)]
pub unsafe fn swap_endianness<S: SIMD256>(s: S, src: *const u32, len: usize, dst: *mut u32) {
    vsimd::bswap::bswap_simd(s, src, len, dst);
}

#[inline(always)]
pub fn is_utf32le_ct<S: SIMD256>(s: S, data: &[u32]) -> bool {
    unsafe {
        let mut y = s.u32x8_splat(0);

        let (mut src, mut len) = slice_parts(data);

        let end = src.add(len / 8 * 8);
        while src < end {
            let x = s.v256_load_unaligned(src.cast::<u8>());
            let a1 = s.v256_xor(x, s.u32x8_splat(0xD800));
            let a2 = s.u32x8_sub(a1, s.u32x8_splat(0x800));
            y = s.u32x8_max(y, a2);

            src = src.add(8);
        }
        len %= 8;

        let m = s.u32x8_splat(0x11_0000 - 0x800 - 1);
        let mut ans = s.v256_all_zero(s.u32x8_lt(m, y));

        ans &= fallback::is_utf32le_ct(slice(src, len));
        ans
    }
}
