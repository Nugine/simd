use vsimd::SIMD256;

#[inline(always)]
pub unsafe fn swap_endianness_fallback(src: *const u32, len: usize, dst: *mut u32) {
    vsimd::bswap::bswap_fallback(src, len, dst);
}

#[inline(always)]
pub unsafe fn swap_endianness_simd<S: SIMD256>(s: S, src: *const u32, len: usize, dst: *mut u32) {
    vsimd::bswap::bswap_simd(s, src, len, dst);
}

// ----------------------------------------------------------------

/// See [`char::from_u32`](core::char::from_u32)
#[inline(always)]
fn is_unicode_scalar_value(x: u32) -> bool {
    (x ^ 0xD800).wrapping_sub(0x800) < (0x11_0000 - 0x800)
}

#[inline]
pub unsafe fn is_utf32le_fallback(mut src: *const u32, len: usize) -> bool {
    let mut flag = true;

    let end = src.add(len);
    while src < end {
        let x = src.read();
        flag &= is_unicode_scalar_value(x);
        src = src.add(1);
    }

    flag
}

#[inline(always)]
pub unsafe fn is_utf32le_simd<S: SIMD256>(s: S, mut src: *const u32, mut len: usize) -> bool {
    let mut y = s.u32x8_splat(0);

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

    ans &= is_utf32le_fallback(src, len);
    ans
}
