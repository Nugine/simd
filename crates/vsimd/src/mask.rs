use crate::isa::{AVX2, NEON, SSE41, WASM128};
use crate::vector::{V128, V256};
use crate::{SIMD128, SIMD256};

use core::ops::Not;

/// x: `{{0xff or 0x00}}x32`
#[inline(always)]
pub fn mask8x16_all<S: SIMD128>(s: S, x: V128) -> bool {
    if is_subtype!(S, SSE41 | WASM128) {
        return s.u8x16_bitmask(x) == u16::MAX;
    }
    if is_subtype!(S, NEON) {
        return s.u8x16_any_zero(x).not();
    }
    unreachable!()
}

#[inline(always)]
pub fn mask8x32_all<S: SIMD256>(s: S, x: V256) -> bool {
    if is_subtype!(S, AVX2) {
        return s.u8x32_bitmask(x) == u32::MAX;
    }
    if is_subtype!(S, SSE41 | WASM128 | NEON) {
        let x = x.to_v128x2();
        let x = s.v128_and(x.0, x.1);
        return mask8x16_all(s, x);
    }
    unreachable!()
}

/// x: `{{0xff or 0x00}}x32`
#[inline(always)]
pub fn mask8x16_any<S: SIMD128>(s: S, x: V128) -> bool {
    if is_subtype!(S, SSE41 | WASM128) {
        return s.u8x16_bitmask(x) != 0;
    }
    if is_subtype!(S, NEON) {
        return s.v128_all_zero(x).not();
    }
    unreachable!()
}

#[inline(always)]
pub fn mask8x32_any<S: SIMD256>(s: S, x: V256) -> bool {
    if is_subtype!(S, AVX2) {
        return s.u8x32_bitmask(x) != 0;
    }
    if is_subtype!(S, SSE41 | WASM128 | NEON) {
        let x = x.to_v128x2();
        let x = s.v128_or(x.0, x.1);
        return mask8x16_any(s, x);
    }
    unreachable!()
}

#[inline(always)]
pub fn u8x16_highbit_all<S: SIMD128>(s: S, x: V128) -> bool {
    if is_subtype!(S, SSE41 | WASM128) {
        return s.u8x16_bitmask(x) == u16::MAX;
    }
    if is_subtype!(S, NEON) {
        if cfg!(target_arch = "arm") {
            return mask8x16_all(s, s.i8x16_lt(x, s.v128_create_zero()));
        }
        if cfg!(target_arch = "aarch64") {
            return s.u8x16_reduce_min(x) >= 0x80;
        }
    }
    unreachable!()
}

#[inline(always)]
pub fn u8x32_highbit_all<S: SIMD256>(s: S, x: V256) -> bool {
    if is_subtype!(S, AVX2) {
        return s.u8x32_bitmask(x) == u32::MAX;
    }
    if is_subtype!(S, SSE41 | WASM128 | NEON) {
        let x = x.to_v128x2();
        let x = s.v128_and(x.0, x.1);
        return u8x16_highbit_all(s, x);
    }
    unreachable!()
}

#[inline(always)]
pub fn u8x16_highbit_any<S: SIMD128>(s: S, x: V128) -> bool {
    if is_subtype!(S, SSE41 | WASM128) {
        return s.u8x16_bitmask(x) != 0;
    }
    if is_subtype!(S, NEON) {
        if cfg!(target_arch = "arm") {
            return mask8x16_any(s, s.i8x16_lt(x, s.v128_create_zero()));
        }
        if cfg!(target_arch = "aarch64") {
            return s.u8x16_reduce_max(x) >= 0x80;
        }
    }
    unreachable!()
}

#[inline(always)]
pub fn u8x32_highbit_any<S: SIMD256>(s: S, x: V256) -> bool {
    if is_subtype!(S, AVX2) {
        return s.u8x32_bitmask(x) != 0;
    }
    if is_subtype!(S, SSE41 | WASM128 | NEON) {
        let x = x.to_v128x2();
        let x = s.v128_or(x.0, x.1);
        return u8x16_highbit_any(s, x);
    }
    unreachable!()
}
