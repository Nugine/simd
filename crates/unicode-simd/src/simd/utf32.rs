use crate::fallback::utf32 as fallback;

use vsimd::pod::align;
use vsimd::tools::unroll;
use vsimd::vector::V256;
use vsimd::SIMD256;

#[inline(always)]
pub unsafe fn swap_endianness<S: SIMD256>(s: S, src: *const u32, len: usize, dst: *mut u32) {
    vsimd::bswap::bswap_simd(s, src, len, dst)
}

#[inline(always)]
pub fn is_utf32le_ct<S: SIMD256>(s: S, data: &[u32]) -> bool {
    let (prefix, middle, suffix) = align::<_, V256>(data);

    let mut ans = fallback::is_utf32le_ct(prefix);

    {
        let mut y = s.u32x8_splat(0);

        unroll(middle, 8, |&x| {
            let a1 = s.v256_xor(x, s.u32x8_splat(0xD800));
            let a2 = s.u32x8_sub(a1, s.u32x8_splat(0x800));
            y = s.u32x8_max(y, a2);
        });

        let m = s.u32x8_splat(0x110000 - 0x800 - 1);
        ans &= s.v256_all_zero(s.u32x8_lt(m, y));
    }

    ans &= fallback::is_utf32le_ct(suffix);

    ans
}
