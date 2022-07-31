use super::SIMD256;

#[inline(always)]
pub fn u16x16_bswap<S: SIMD256>(s: S, a: S::V256) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    s.v256_from_v128x2(s.u16x8_bswap(a.0), s.u16x8_bswap(a.1))
}

#[inline(always)]
pub fn u32x8_bswap<S: SIMD256>(s: S, a: S::V256) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    s.v256_from_v128x2(s.u32x4_bswap(a.0), s.u32x4_bswap(a.1))
}

#[inline(always)]
pub fn u64x4_bswap<S: SIMD256>(s: S, a: S::V256) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    s.v256_from_v128x2(s.u64x2_bswap(a.0), s.u64x2_bswap(a.1))
}
