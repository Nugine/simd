use super::SIMD256;

#[inline(always)]
fn vertical_map<S: SIMD256>(
    s: S,
    a: S::V256,
    b: S::V256,
    f: impl FnOnce((S::V128, S::V128), (S::V128, S::V128)) -> (S::V128, S::V128),
) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = s.v256_to_v128x2(b);
    let (c0, c1) = f(a, b);
    s.v256_from_v128x2(c0, c1)
}

#[inline(always)]
pub fn u8x32_add<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vertical_map(s, a, b, |a, b| {
        (s.u8x16_add(a.0, b.0), s.u8x16_add(a.1, b.1))
    })
}

#[inline(always)]
pub fn u32x8_add<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vertical_map(s, a, b, |a, b| {
        (s.u32x4_add(a.0, b.0), s.u32x4_add(a.1, b.1))
    })
}

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
