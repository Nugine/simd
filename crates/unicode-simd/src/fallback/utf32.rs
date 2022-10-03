use vsimd::tools::unroll;

#[inline]
pub unsafe fn swap_endianness(src: *const u32, len: usize, dst: *mut u32) {
    vsimd::bswap::bswap_fallback(src, len, dst);
}

/// See [`char::from_u32`]
#[inline]
pub fn is_utf32le_ct(data: &[u32]) -> bool {
    let mut flag = false;
    unroll(data, 8, |&x| {
        flag |= (x ^ 0xD800).wrapping_sub(0x800) >= (0x11_0000 - 0x800);
    });
    !flag
}
