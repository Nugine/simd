use crate::{NEON, SIMD128, SIMD256, SSE41, V128, V256, WASM128};

#[inline(always)]
pub fn u8x16_lookup<S: SIMD128>(s: S, lut: V128, x: V128) -> V128 {
    if is_subtype!(S, SSE41) {
        return s.u8x16_swizzle(lut, x);
    }

    if is_subtype!(S, NEON | WASM128) {
        let idx = s.v128_and(x, s.u8x16_splat(0x8f));
        return s.u8x16_swizzle(lut, idx);
    }

    unreachable!()
}

#[inline(always)]
pub fn u8x16x2_lookup<S: SIMD256>(s: S, lut: V256, x: V256) -> V256 {
    if is_subtype!(S, SSE41) {
        return s.u8x16x2_swizzle(lut, x);
    }

    if is_subtype!(S, NEON | WASM128) {
        let idx = s.v256_and(x, s.u8x32_splat(0x8f));
        return s.u8x16x2_swizzle(lut, idx);
    }

    unreachable!()
}
