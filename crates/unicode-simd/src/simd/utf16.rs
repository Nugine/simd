use vsimd::SIMD256;

#[inline(always)]
pub unsafe fn swap_endianness<S: SIMD256>(s: S, src: *const u16, len: usize, dst: *mut u16) {
    vsimd::bswap::bswap_simd(s, src, len, dst)
}
