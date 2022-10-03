#[inline]
pub unsafe fn swap_endianness(src: *const u16, len: usize, dst: *mut u16) {
    vsimd::bswap::bswap_fallback(src, len, dst);
}
