use simd_abstraction::isa::{SimdLoad, SIMD256};
use simd_abstraction::scalar::align32;
use simd_abstraction::tools::unroll;

/// See [`char::from_u32`]
#[inline]
pub fn is_utf32le_ct_fallback(data: &[u32]) -> bool {
    let mut flag = false;
    unroll(data, 8, |&x| {
        flag |= (x ^ 0xD800).wrapping_sub(0x800) >= (0x110000 - 0x800);
    });
    !flag
}

#[inline]
pub fn is_utf32le_ct_simd<S: SIMD256>(s: S, data: &[u32]) -> bool {
    let (prefix, middle, suffix) = align32(data);

    let mut ans = is_utf32le_ct_fallback(prefix);

    {
        let mut y = s.u32x8_splat(0);

        unroll(middle, 8, |chunk| {
            let x = s.load(chunk);
            let a1 = s.v256_xor(x, s.u32x8_splat(0xD800));
            let a2 = s.u32x8_sub(a1, s.u32x8_splat(0x800));
            y = s.u32x8_max(y, a2);
        });

        let m = s.u32x8_splat(0x110000 - 0x800 - 1);
        ans &= s.v256_all_zero(s.u32x8_lt(m, y));
    }

    ans &= is_utf32le_ct_fallback(suffix);

    ans
}

#[inline]
pub unsafe fn utf32_swap_endianness_raw_fallback(src: *const u32, len: usize, dst: *mut u32) {
    crate::sa_bswap::bswap_raw_fallback(src, len, dst)
}

#[inline]
pub unsafe fn utf32_swap_endianness_raw_simd<S: SIMD256>(
    s: S,
    src: *const u32,
    len: usize,
    dst: *mut u32,
) {
    crate::sa_bswap::bswap_raw_simd(s, src, len, dst)
}
