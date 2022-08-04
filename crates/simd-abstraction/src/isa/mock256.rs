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
fn double<S: SIMD256>(s: S, f: impl FnOnce() -> S::V128) -> S::V256 {
    let a = f();
    s.v256_from_v128x2(a, a)
}

#[inline(always)]
pub fn u8x32_splat<S: SIMD256>(s: S, x: u8) -> S::V256 {
    double(s, || s.u8x16_splat(x))
}

#[inline(always)]
pub fn u16x16_splat<S: SIMD256>(s: S, x: u16) -> S::V256 {
    double(s, || s.u16x8_splat(x))
}

#[inline(always)]
pub fn u32x8_splat<S: SIMD256>(s: S, x: u32) -> S::V256 {
    double(s, || s.u32x4_splat(x))
}

#[inline(always)]
pub fn u64x4_splat<S: SIMD256>(s: S, x: u64) -> S::V256 {
    double(s, || s.u64x2_splat(x))
}

#[inline(always)]
pub fn i8x32_splat<S: SIMD256>(s: S, x: i8) -> S::V256 {
    double(s, || s.i8x16_splat(x))
}

#[inline(always)]
pub fn i16x16_splat<S: SIMD256>(s: S, x: i16) -> S::V256 {
    double(s, || s.i16x8_splat(x))
}

#[inline(always)]
pub fn i32x8_splat<S: SIMD256>(s: S, x: i32) -> S::V256 {
    double(s, || s.i32x4_splat(x))
}

#[inline(always)]
pub fn i64x4_splat<S: SIMD256>(s: S, x: i64) -> S::V256 {
    double(s, || s.i64x2_splat(x))
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
pub fn u16x16_shl<S: SIMD256, const IMM8: i32>(s: S, a: S::V256) -> S::V256 {
    vmap(s, a, S::u16x8_shl::<IMM8>)
}

#[inline(always)]
pub fn u32x8_shl<S: SIMD256, const IMM8: i32>(s: S, a: S::V256) -> S::V256 {
    vmap(s, a, S::u32x4_shl::<IMM8>)
}

#[inline(always)]
pub fn u16x16_shr<S: SIMD256, const IMM8: i32>(s: S, a: S::V256) -> S::V256 {
    vmap(s, a, S::u16x8_shr::<IMM8>)
}

#[inline(always)]
pub fn u32x8_shr<S: SIMD256, const IMM8: i32>(s: S, a: S::V256) -> S::V256 {
    vmap(s, a, S::u32x4_shr::<IMM8>)
}

#[inline(always)]
pub fn u8x32_lt<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u8x16_lt)
}

#[inline(always)]
pub fn u16x16_lt<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u16x8_lt)
}

#[inline(always)]
pub fn u32x8_lt<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u32x4_lt)
}

#[inline(always)]
pub fn i8x32_lt<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::i8x16_lt)
}

#[inline(always)]
pub fn i16x16_lt<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::i16x8_lt)
}

#[inline(always)]
pub fn i32x8_lt<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::i32x4_lt)
}

#[inline(always)]
pub fn u8x32_max<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u8x16_max)
}

#[inline(always)]
pub fn u16x16_max<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u16x8_max)
}

#[inline(always)]
pub fn u32x8_max<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u32x4_max)
}

#[inline(always)]
pub fn i8x32_max<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::i8x16_max)
}

#[inline(always)]
pub fn i16x16_max<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::i16x8_max)
}

#[inline(always)]
pub fn i32x8_max<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::i32x4_max)
}

#[inline(always)]
pub fn u8x32_min<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u8x16_min)
}

#[inline(always)]
pub fn u16x16_min<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u16x8_min)
}

#[inline(always)]
pub fn u32x8_min<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::u32x4_min)
}

#[inline(always)]
pub fn i8x32_min<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::i8x16_min)
}

#[inline(always)]
pub fn i16x16_min<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::i16x8_min)
}

#[inline(always)]
pub fn i32x8_min<S: SIMD256>(s: S, a: S::V256, b: S::V256) -> S::V256 {
    vmerge(s, a, b, S::i32x4_min)
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
