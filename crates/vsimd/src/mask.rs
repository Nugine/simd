use crate::{AVX2, NEON, SIMD128, SIMD256, SSE41, V128, V256, WASM128};

use core::ops::Not;

/// x: `{{0xff or 0x00}}x32`
#[inline(always)]
pub fn mask8x32_all<S: SIMD256>(s: S, x: V256) -> bool {
    if is_subtype!(S, AVX2) {
        return s.u8x32_bitmask(x) == u32::MAX;
    }
    if is_subtype!(S, SSE41 | WASM128) {
        let x = x.to_v128x2();
        return s.u8x16_bitmask(s.v128_and(x.0, x.1)) == u16::MAX;
    }
    if is_subtype!(S, NEON) {
        return s.u8x32_any_zero(x).not();
    }
    {
        unreachable!()
    }
}

#[inline(always)]
pub fn mask8x16_all<S: SIMD128>(s: S, x: V128) -> bool {
    if is_subtype!(S, SSE41 | WASM128) {
        return s.u8x16_bitmask(x) == u16::MAX;
    }
    if is_subtype!(S, NEON) {
        return s.u8x16_any_zero(x).not();
    }
    {
        unreachable!()
    }
}

/// x: `{{0xff or 0x00}}x32`
#[inline(always)]
pub fn mask8x32_any<S: SIMD256>(s: S, x: V256) -> bool {
    if is_subtype!(S, AVX2) {
        return s.u8x32_bitmask(x) != 0;
    }
    if is_subtype!(S, SSE41 | WASM128) {
        let x = x.to_v128x2();
        return s.u8x16_bitmask(s.v128_or(x.0, x.1)) != 0;
    }
    if is_subtype!(S, NEON) {
        return s.v256_all_zero(x).not();
    }
    {
        unreachable!()
    }
}

#[inline(always)]
pub fn mask8x16_any<S: SIMD128>(s: S, x: V128) -> bool {
    if is_subtype!(S, SSE41 | WASM128) {
        return s.u8x16_bitmask(x) != 0;
    }
    if is_subtype!(S, NEON) {
        return s.v128_all_zero(x).not();
    }
    {
        unreachable!()
    }
}
