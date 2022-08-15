use simd_abstraction::isa::SIMD256;

#[inline]
pub unsafe fn utf16_swap_endianness_fallback(src: *const u16, len: usize, dst: *mut u16) {
    crate::sa_bswap::bswap_fallback(src, len, dst)
}

#[inline]
pub unsafe fn utf16_swap_endianness_simd<S: SIMD256>(s: S, src: *const u16, len: usize, dst: *mut u16) {
    crate::sa_bswap::bswap_simd(s, src, len, dst)
}
