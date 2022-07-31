use super::SIMD256;

#[inline(always)]
fn vmap<S: SIMD256>(s: S, a: S::V256, f: impl Fn(S, S::V128) -> S::V128) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = (f(s, a.0), f(s, a.1));
    s.v256_from_v128x2(b.0, b.1)
}

#[inline(always)]
fn vmerge<S: SIMD256>(s: S, a: S::V256, b: S::V256, f: impl Fn(S, S::V128, S::V128) -> S::V128) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = s.v256_to_v128x2(b);
    let c = (f(s, a.0, b.0), f(s, a.1, b.1));
    s.v256_from_v128x2(c.0, c.1)
}

#[inline(always)]
pub fn u8x32_add<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u8x16_add)
}

#[inline(always)]
pub fn u16x16_add<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u16x8_add)
}

#[inline(always)]
pub fn u32x8_add<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u32x4_add)
}

#[inline(always)]
pub fn u64x4_add<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u64x2_add)
}

#[inline(always)]
pub fn u8x32_sub<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u8x16_sub)
}

#[inline(always)]
pub fn u16x16_sub<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u16x8_sub)
}

#[inline(always)]
pub fn u32x8_sub<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u32x4_sub)
}

#[inline(always)]
pub fn u64x4_sub<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u64x2_sub)
}

#[inline(always)]
pub fn u16x16_bswap<S: SIMD256>(s: S, a: S::V256) -> S::V256 {
    vmap(s, a, S::u16x8_bswap)
}

#[inline(always)]
pub fn u32x8_bswap<S: SIMD256>(s: S, a: S::V256) -> S::V256 {
    vmap(s, a, S::u32x4_bswap)
}

#[inline(always)]
pub fn u64x4_bswap<S: SIMD256>(s: S, a: S::V256) -> S::V256 {
    vmap(s, a, S::u64x2_bswap)
}
