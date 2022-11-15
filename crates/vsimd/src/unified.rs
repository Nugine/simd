#![allow(clippy::collapsible_if, clippy::too_many_lines)]

use crate::isa::InstructionSet;
use crate::tools::transmute_copy as tc;
use crate::vector::{V128, V256};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::isa::{AVX2, SSE2, SSE41};

#[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
use crate::isa::NEON;

#[cfg(target_arch = "wasm32")]
use crate::isa::WASM128;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(all(feature = "unstable", target_arch = "arm"))]
use core::arch::arm::*;

#[cfg(all(feature = "unstable", target_arch = "aarch64"))]
use core::arch::aarch64::*;

#[cfg(target_arch = "wasm32")]
use core::arch::wasm32::*;

#[inline(always)]
pub fn splat<S: InstructionSet, T: Copy + 'static, V: Copy + 'static>(s: S, x: T) -> V {
    if is_same_type!(V, V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, AVX2) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&_mm256_set1_epi8(tc(&x))) };
            }
        }
        {
            let c = splat::<S, T, V128>(s, x).x2();
            return unsafe { tc(&c) };
        }
    }
    if is_same_type!(V, V128) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE2) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&_mm_set1_epi8(tc(&x))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&_mm_set1_epi16(tc(&x))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&_mm_set1_epi32(tc(&x))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&_mm_set1_epi64x(tc(&x))) };
            }
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(S, NEON) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&vld1q_dup_u8(&tc(&x))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&vld1q_dup_u16(&tc(&x))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&vld1q_dup_u32(&tc(&x))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&vld1q_dup_u64(&tc(&x))) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(S, WASM128) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&u8x16_splat(tc(&x))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&u16x8_splat(tc(&x))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&u32x4_splat(tc(&x))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&u64x2_splat(tc(&x))) };
            }
        }
    }
    {
        let _ = (s, x);
        unreachable!()
    }
}

#[inline(always)]
pub fn add<S: InstructionSet, T: Copy + 'static, V: Copy + 'static>(s: S, a: V, b: V) -> V {
    if is_same_type!(V, V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, AVX2) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&_mm256_add_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&_mm256_add_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&_mm256_add_epi32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&_mm256_add_epi64(tc(&a), tc(&b))) };
            }
        }
        {
            let (a, b): (V256, V256) = unsafe { (tc(&a), tc(&b)) };
            let (a, b) = (a.to_v128x2(), b.to_v128x2());
            let c0 = add::<S, T, V128>(s, a.0, b.0);
            let c1 = add::<S, T, V128>(s, a.1, b.1);
            return unsafe { tc(&V256::from_v128x2((c0, c1))) };
        }
    }
    if is_same_type!(V, V128) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE2) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&_mm_add_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&_mm_add_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&_mm_add_epi32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&_mm_add_epi64(tc(&a), tc(&b))) };
            }
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(S, NEON) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&vaddq_u8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&vaddq_u16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&vaddq_u32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&vaddq_u64(tc(&a), tc(&b))) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(S, WASM128) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&u8x16_add(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&u16x8_add(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&u32x4_add(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&u64x2_add(tc(&a), tc(&b))) };
            }
        }
    }
    {
        let _ = (s, a, b);
        unreachable!()
    }
}

#[inline(always)]
pub fn sub<S: InstructionSet, T: Copy + 'static, V: Copy + 'static>(s: S, a: V, b: V) -> V {
    if is_same_type!(V, V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, AVX2) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&_mm256_sub_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&_mm256_sub_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&_mm256_sub_epi32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&_mm256_sub_epi64(tc(&a), tc(&b))) };
            }
        }
        {
            let (a, b): (V256, V256) = unsafe { (tc(&a), tc(&b)) };
            let (a, b) = (a.to_v128x2(), b.to_v128x2());
            let c0 = sub::<S, T, V128>(s, a.0, b.0);
            let c1 = sub::<S, T, V128>(s, a.1, b.1);
            return unsafe { tc(&V256::from_v128x2((c0, c1))) };
        }
    }
    if is_same_type!(V, V128) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE2) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&_mm_sub_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&_mm_sub_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&_mm_sub_epi32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&_mm_sub_epi64(tc(&a), tc(&b))) };
            }
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(S, NEON) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&vsubq_u8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&vsubq_u16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&vsubq_u32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&vsubq_u64(tc(&a), tc(&b))) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(S, WASM128) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&u8x16_sub(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&u16x8_sub(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&u32x4_sub(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&u64x2_sub(tc(&a), tc(&b))) };
            }
        }
    }
    {
        let _ = (s, a, b);
        unreachable!()
    }
}

#[inline(always)]
pub fn eq<S: InstructionSet, T: Copy + 'static, V: Copy + 'static>(s: S, a: V, b: V) -> V {
    if is_same_type!(V, V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, AVX2) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&_mm256_cmpeq_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&_mm256_cmpeq_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&_mm256_cmpeq_epi32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&_mm256_cmpeq_epi64(tc(&a), tc(&b))) };
            }
        }
        {
            let (a, b): (V256, V256) = unsafe { (tc(&a), tc(&b)) };
            let (a, b) = (a.to_v128x2(), b.to_v128x2());
            let c0 = eq::<S, T, V128>(s, a.0, b.0);
            let c1 = eq::<S, T, V128>(s, a.1, b.1);
            return unsafe { tc(&V256::from_v128x2((c0, c1))) };
        }
    }
    if is_same_type!(V, V128) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE2) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&_mm_cmpeq_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&_mm_cmpeq_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&_mm_cmpeq_epi32(tc(&a), tc(&b))) };
            }
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(S, NEON) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&vceqq_u8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&vceqq_u16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&vceqq_u32(tc(&a), tc(&b))) };
            }
            #[cfg(target_arch = "aarch64")]
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&vceqq_u64(tc(&a), tc(&b))) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(S, WASM128) {
            if is_same_type!(T, u8 | i8) {
                return unsafe { tc(&u8x16_eq(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16 | i16) {
                return unsafe { tc(&u16x8_eq(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32 | i32) {
                return unsafe { tc(&u32x4_eq(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u64 | i64) {
                return unsafe { tc(&u64x2_eq(tc(&a), tc(&b))) };
            }
        }
    }
    {
        let _ = (s, a, b);
        unreachable!()
    }
}

#[inline(always)]
pub fn lt<S: InstructionSet, T: Copy + 'static, V: Copy + 'static>(s: S, a: V, b: V) -> V {
    if is_same_type!(V, V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, AVX2) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm256_cmpgt_epi8(tc(&b), tc(&a))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm256_cmpgt_epi16(tc(&b), tc(&a))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&_mm256_cmpgt_epi32(tc(&b), tc(&a))) };
            }
            if is_same_type!(T, i64) {
                return unsafe { tc(&_mm256_cmpgt_epi64(tc(&b), tc(&a))) };
            }
            if is_same_type!(T, u8) {
                return unsafe {
                    let (a, b) = (tc(&a), tc(&b));
                    let c = _mm256_cmpeq_epi8(a, _mm256_max_epu8(a, b));
                    tc(&_mm256_xor_si256(c, _mm256_cmpeq_epi8(a, a)))
                };
            }
            if is_same_type!(T, u16) {
                return unsafe {
                    let (a, b) = (tc(&a), tc(&b));
                    let c = _mm256_cmpeq_epi16(a, _mm256_max_epu16(a, b));
                    tc(&_mm256_xor_si256(c, _mm256_cmpeq_epi16(a, a)))
                };
            }
            if is_same_type!(T, u32) {
                return unsafe {
                    let (a, b) = (tc(&a), tc(&b));
                    let c = _mm256_cmpeq_epi32(a, _mm256_max_epu32(a, b));
                    tc(&_mm256_xor_si256(c, _mm256_cmpeq_epi32(a, a)))
                };
            }
        }
        {
            let (a, b): (V256, V256) = unsafe { (tc(&a), tc(&b)) };
            let (a, b) = (a.to_v128x2(), b.to_v128x2());
            let c0 = lt::<S, T, V128>(s, a.0, b.0);
            let c1 = lt::<S, T, V128>(s, a.1, b.1);
            return unsafe { tc(&V256::from_v128x2((c0, c1))) };
        }
    }
    if is_same_type!(V, V128) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE2) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm_cmplt_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm_cmplt_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&_mm_cmplt_epi32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe {
                    let (a, b) = (tc(&a), tc(&b));
                    let c = _mm_cmpeq_epi8(a, _mm_max_epu8(a, b));
                    tc(&_mm_xor_si128(c, _mm_cmpeq_epi8(a, a)))
                };
            }
            if is_same_type!(T, u16) {
                return unsafe {
                    let m = _mm_set1_epi16(i16::MIN);
                    let a = _mm_xor_si128(tc(&a), m);
                    let b = _mm_xor_si128(tc(&b), m);
                    tc(&_mm_cmplt_epi16(a, b))
                };
            }
            if is_same_type!(T, u32) {
                return unsafe {
                    let m = _mm_set1_epi32(i32::MIN);
                    let a = _mm_xor_si128(tc(&a), m);
                    let b = _mm_xor_si128(tc(&b), m);
                    tc(&_mm_cmplt_epi32(a, b))
                };
            }
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(S, NEON) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&vcltq_s8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&vcltq_s16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&vcltq_s32(tc(&a), tc(&b))) };
            }
            #[cfg(target_arch = "aarch64")]
            if is_same_type!(T, i64) {
                return unsafe { tc(&vcltq_s64(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&vcltq_u8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&vcltq_u16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&vcltq_u32(tc(&a), tc(&b))) };
            }
            #[cfg(target_arch = "aarch64")]
            if is_same_type!(T, u64) {
                return unsafe { tc(&vcltq_u64(tc(&a), tc(&b))) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(S, WASM128) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&i8x16_lt(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&i16x8_lt(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&i32x4_lt(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i64) {
                return unsafe { tc(&i64x2_lt(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&u8x16_lt(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&u16x8_lt(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&u32x4_lt(tc(&a), tc(&b))) };
            }
            // if is_same_type!(T, u64) {
            //     return unsafe { tc(&u64x2_lt(tc(&a), tc(&b))) };
            // }
        }
    }
    {
        let _ = (s, a, b);
        unreachable!()
    }
}

#[inline(always)]
pub fn add_sat<S: InstructionSet, T: Copy + 'static, V: Copy + 'static>(s: S, a: V, b: V) -> V {
    if is_same_type!(V, V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, AVX2) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm256_adds_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm256_adds_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&_mm256_adds_epu8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&_mm256_adds_epu16(tc(&a), tc(&b))) };
            }
        }
        {
            let (a, b): (V256, V256) = unsafe { (tc(&a), tc(&b)) };
            let (a, b) = (a.to_v128x2(), b.to_v128x2());
            let c0 = add_sat::<S, T, V128>(s, a.0, b.0);
            let c1 = add_sat::<S, T, V128>(s, a.1, b.1);
            return unsafe { tc(&V256::from_v128x2((c0, c1))) };
        }
    }
    if is_same_type!(V, V128) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE2) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm_adds_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm_adds_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&_mm_adds_epu8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&_mm_adds_epu16(tc(&a), tc(&b))) };
            }
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(S, NEON) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&vqaddq_s8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&vqaddq_s16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&vqaddq_s32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&vqaddq_u8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&vqaddq_u16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&vqaddq_u32(tc(&a), tc(&b))) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(S, WASM128) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&i8x16_add_sat(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&i16x8_add_sat(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&u8x16_add_sat(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&u16x8_add_sat(tc(&a), tc(&b))) };
            }
        }
    }
    {
        let _ = (s, a, b);
        unreachable!()
    }
}

#[inline(always)]
pub fn sub_sat<S: InstructionSet, T: Copy + 'static, V: Copy + 'static>(s: S, a: V, b: V) -> V {
    if is_same_type!(V, V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, AVX2) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm256_subs_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm256_subs_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&_mm256_subs_epu8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&_mm256_subs_epu16(tc(&a), tc(&b))) };
            }
        }
        {
            let (a, b): (V256, V256) = unsafe { (tc(&a), tc(&b)) };
            let (a, b) = (a.to_v128x2(), b.to_v128x2());
            let c0 = sub_sat::<S, T, V128>(s, a.0, b.0);
            let c1 = sub_sat::<S, T, V128>(s, a.1, b.1);
            return unsafe { tc(&V256::from_v128x2((c0, c1))) };
        }
    }
    if is_same_type!(V, V128) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE2) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm_subs_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm_subs_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&_mm_subs_epu8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&_mm_subs_epu16(tc(&a), tc(&b))) };
            }
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(S, NEON) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&vqsubq_s8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&vqsubq_s16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&vqsubq_s32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&vqsubq_u8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&vqsubq_u16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&vqsubq_u32(tc(&a), tc(&b))) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(S, WASM128) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&i8x16_sub_sat(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&i16x8_sub_sat(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&u8x16_sub_sat(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&u16x8_sub_sat(tc(&a), tc(&b))) };
            }
        }
    }
    {
        let _ = (s, a, b);
        unreachable!()
    }
}

pub fn max<S: InstructionSet, T: Copy + 'static, V: Copy + 'static>(s: S, a: V, b: V) -> V {
    if is_same_type!(V, V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, AVX2) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm256_max_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm256_max_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&_mm256_max_epi32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&_mm256_max_epu8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&_mm256_max_epu16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&_mm256_max_epu32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f32) {
                return unsafe { tc(&_mm256_max_ps(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f64) {
                return unsafe { tc(&_mm256_max_pd(tc(&a), tc(&b))) };
            }
        }
        {
            let (a, b): (V256, V256) = unsafe { (tc(&a), tc(&b)) };
            let (a, b) = (a.to_v128x2(), b.to_v128x2());
            let c0 = max::<S, T, V128>(s, a.0, b.0);
            let c1 = max::<S, T, V128>(s, a.1, b.1);
            return unsafe { tc(&V256::from_v128x2((c0, c1))) };
        }
    }
    if is_same_type!(V, V128) {
        #[cfg(miri)]
        {
            if is_same_type!(T, u8) {
                return unsafe { tc(&crate::simulation::u8x16_max(tc(&a), tc(&b))) };
            }
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE41) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm_max_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&_mm_max_epi32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&_mm_max_epu16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&_mm_max_epu32(tc(&a), tc(&b))) };
            }
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE2) {
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm_max_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&_mm_max_epu8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f32) {
                return unsafe { tc(&_mm_max_ps(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f64) {
                return unsafe { tc(&_mm_max_pd(tc(&a), tc(&b))) };
            }
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(S, NEON) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&vmaxq_s8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&vmaxq_s16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&vmaxq_s32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&vmaxq_u8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&vmaxq_u16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&vmaxq_u32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f32) {
                return unsafe { tc(&vmaxq_f32(tc(&a), tc(&b))) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(S, WASM128) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&i8x16_max(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&i16x8_max(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&i32x4_max(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&u8x16_max(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&u16x8_max(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&u32x4_max(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f32) {
                return unsafe { tc(&f32x4_max(tc(&a), tc(&b))) };
            }
        }
    }
    {
        let _ = (s, a, b);
        unreachable!()
    }
}

#[inline(always)]
pub fn min<S: InstructionSet, T: Copy + 'static, V: Copy + 'static>(s: S, a: V, b: V) -> V {
    if is_same_type!(V, V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, AVX2) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm256_min_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm256_min_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&_mm256_min_epi32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&_mm256_min_epu8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&_mm256_min_epu16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&_mm256_min_epu32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f32) {
                return unsafe { tc(&_mm256_min_ps(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f64) {
                return unsafe { tc(&_mm256_min_pd(tc(&a), tc(&b))) };
            }
        }
        {
            let (a, b): (V256, V256) = unsafe { (tc(&a), tc(&b)) };
            let (a, b) = (a.to_v128x2(), b.to_v128x2());
            let c0 = min::<S, T, V128>(s, a.0, b.0);
            let c1 = min::<S, T, V128>(s, a.1, b.1);
            return unsafe { tc(&V256::from_v128x2((c0, c1))) };
        }
    }
    if is_same_type!(V, V128) {
        #[cfg(miri)]
        {
            if is_same_type!(T, u8) {
                return unsafe { tc(&crate::simulation::u8x16_min(tc(&a), tc(&b))) };
            }
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE41) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&_mm_min_epi8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&_mm_min_epi32(tc(&a), tc(&b))) };
            }

            if is_same_type!(T, u16) {
                return unsafe { tc(&_mm_min_epu16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&_mm_min_epu32(tc(&a), tc(&b))) };
            }
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(S, SSE2) {
            if is_same_type!(T, i16) {
                return unsafe { tc(&_mm_min_epi16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&_mm_min_epu8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f32) {
                return unsafe { tc(&_mm_min_ps(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f64) {
                return unsafe { tc(&_mm_min_pd(tc(&a), tc(&b))) };
            }
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(S, NEON) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&vminq_s8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&vminq_s16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&vminq_s32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&vminq_u8(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&vminq_u16(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&vminq_u32(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f32) {
                return unsafe { tc(&vminq_f32(tc(&a), tc(&b))) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(S, WASM128) {
            if is_same_type!(T, i8) {
                return unsafe { tc(&i8x16_min(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i16) {
                return unsafe { tc(&i16x8_min(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, i32) {
                return unsafe { tc(&i32x4_min(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u8) {
                return unsafe { tc(&u8x16_min(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u16) {
                return unsafe { tc(&u16x8_min(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, u32) {
                return unsafe { tc(&u32x4_min(tc(&a), tc(&b))) };
            }
            if is_same_type!(T, f32) {
                return unsafe { tc(&f32x4_min(tc(&a), tc(&b))) };
            }
        }
    }
    {
        let _ = (s, a, b);
        unreachable!()
    }
}
