use crate::{InstructionSet, NEON, SSE41, V128, WASM128};

use core::mem::transmute as t;

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

pub unsafe trait SIMD128: InstructionSet {
    #[inline(always)]
    unsafe fn v128_load(self, addr: *const u8) -> V128 {
        debug_assert_ptr_align!(addr, 16);

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return t(_mm_load_si128(addr.cast()));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return self.v128_load_unaligned(addr);
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return self.v128_load_unaligned(addr);
        }
        {
            let _ = addr;
            unreachable!()
        }
    }

    #[inline(always)]
    unsafe fn v128_load_unaligned(self, addr: *const u8) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return t(_mm_loadu_si128(addr.cast()));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return t(vld1q_u8(addr.cast()));
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return t(v128_load(addr.cast()));
        }
        {
            let _ = addr;
            unreachable!()
        }
    }

    #[inline(always)]
    unsafe fn v128_store(self, addr: *mut u8, a: V128) {
        debug_assert_ptr_align!(addr, 16);

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return _mm_store_si128(addr.cast(), t(a));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return self.v128_store_unaligned(addr, a);
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return self.v128_store_unaligned(addr, a);
        }
        {
            let _ = (addr, a);
            unreachable!()
        }
    }

    #[inline(always)]
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: V128) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return _mm_storeu_si128(addr.cast(), t(a));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return vst1q_u8(addr.cast(), t(a));
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return v128_store(addr.cast(), t(a));
        }
        {
            let _ = (addr, a);
            unreachable!()
        }
    }

    #[inline(always)]
    fn v128_create_zero(self) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_setzero_si128()) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vdupq_n_u8(0)) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_splat(0)) };
        }
        {
            unreachable!()
        }
    }

    #[inline(always)]
    fn v128_not(self, a: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return self.v128_xor(a, self.u8x16_eq(a, a));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vmvnq_u8(t(a))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(v128_not(t(a))) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn v128_and(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_and_si128(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vandq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(v128_and(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn v128_or(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_or_si128(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vorrq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(v128_or(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn v128_xor(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_xor_si128(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(veorq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(v128_xor(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn v128_andnot(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_andnot_si128(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vbicq_u8(t(b), t(a))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(v128_andnot(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn v128_all_zero(self, a: V128) -> bool {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe {
                let a = t(a);
                _mm_testz_si128(a, a) != 0
            };
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe {
                let a = t(a);
                let a = vorr_u64(vget_low_u64(a), vget_high_u64(a));
                vget_lane_u64::<0>(a) == 0
            };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { vmaxvq_u8(t(a)) == 0 };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { !v128_any_true(t(a)) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_splat(self, x: u8) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_set1_epi8(x as i8)) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vdupq_n_u8(x)) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_splat(x)) };
        }
        {
            let _ = x;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_splat(self, x: u16) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_set1_epi16(x as i16)) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vdupq_n_u16(x)) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_splat(x)) };
        }
        {
            let _ = x;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_splat(self, x: u32) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_set1_epi32(x as i32)) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vdupq_n_u32(x)) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u32x4_splat(x)) };
        }
        {
            let _ = x;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u64x2_splat(self, x: u64) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_set1_epi64x(x as i64)) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vdupq_n_u64(x)) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u64x2_splat(x)) };
        }
        {
            let _ = x;
            unreachable!()
        }
    }

    #[inline(always)]
    fn i8x16_splat(self, x: i8) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_set1_epi8(x as i8)) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vdupq_n_s8(x)) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i8x16_splat(x)) };
        }
        {
            let _ = x;
            unreachable!()
        }
    }

    #[inline(always)]
    fn i16x8_splat(self, x: i16) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_set1_epi16(x as i16)) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vdupq_n_s16(x)) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i16x8_splat(x)) };
        }
        {
            let _ = x;
            unreachable!()
        }
    }

    #[inline(always)]
    fn i32x4_splat(self, x: i32) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_set1_epi32(x as i32)) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vdupq_n_s32(x)) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i32x4_splat(x)) };
        }
        {
            let _ = x;
            unreachable!()
        }
    }

    #[inline(always)]
    fn i64x2_splat(self, x: i64) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_set1_epi64x(x as i64)) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vdupq_n_s64(x)) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i64x2_splat(x)) };
        }
        {
            let _ = x;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_add(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_add_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vaddq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_add(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_add(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_add_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vaddq_u16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_add(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_add(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_add_epi32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vaddq_u32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u32x4_add(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u64x2_add(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_add_epi64(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vaddq_u64(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u64x2_add(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_sub(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_sub_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vsubq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_sub(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_sub(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_sub_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vsubq_u16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_sub(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_sub(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_sub_epi32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vsubq_u32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u32x4_sub(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u64x2_sub(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_sub_epi64(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vsubq_u64(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u64x2_sub(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_sub_sat(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_subs_epu8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vqsubq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_sub_sat(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_sub_sat(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_subs_epu16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vqsubq_u16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_sub_sat(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i8x16_sub_sat(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_subs_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vqsubq_s8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i8x16_sub_sat(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i16x8_sub_sat(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_subs_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vqsubq_s16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i16x8_sub_sat(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i16x8_mul_lo(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_mullo_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vmulq_s16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i16x8_mul(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i32x4_mul_lo(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_mullo_epi32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vmulq_s32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i32x4_mul(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_shl<const IMM8: i32>(self, a: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_slli_epi16::<IMM8>(t(a))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vshlq_n_u16::<IMM8>(t(a))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_shl(t(a), IMM8 as u32)) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_shl<const IMM8: i32>(self, a: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_slli_epi32::<IMM8>(t(a))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vshlq_n_u32::<IMM8>(t(a))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u32x4_shl(t(a), IMM8 as u32)) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_shr<const IMM8: i32>(self, a: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_srli_epi16::<IMM8>(t(a))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vshrq_n_u16::<IMM8>(t(a))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_shr(t(a), IMM8 as u32)) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_shr<const IMM8: i32>(self, a: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_srli_epi32::<IMM8>(t(a))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vshrq_n_u32::<IMM8>(t(a))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u32x4_shr(t(a), IMM8 as u32)) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_eq(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_cmpeq_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vceqq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_eq(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_eq(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_cmpeq_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vceqq_u16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_eq(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_eq(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_cmpeq_epi32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vceqq_u32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u32x4_eq(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_lt(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return {
                let shift = self.u8x16_splat(u8::MAX / 2);
                let a = self.u8x16_sub(a, shift);
                let b = self.u8x16_sub(b, shift);
                self.i8x16_lt(a, b)
            };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vcltq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_lt(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_lt(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return {
                let shift = self.u16x8_splat(u16::MAX / 2);
                let a = self.u16x8_sub(a, shift);
                let b = self.u16x8_sub(b, shift);
                self.i16x8_lt(a, b)
            };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vcltq_u16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_lt(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_lt(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return {
                let shift = self.u32x4_splat(u32::MAX / 2);
                let a = self.u32x4_sub(a, shift);
                let b = self.u32x4_sub(b, shift);
                self.i32x4_lt(a, b)
            };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vcltq_u32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u32x4_lt(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i8x16_lt(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_cmplt_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vcltq_s8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i8x16_lt(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i16x8_lt(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_cmplt_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vcltq_s16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i16x8_lt(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i32x4_lt(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_cmplt_epi32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vcltq_s32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i32x4_lt(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_max(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_max_epu8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vmaxq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_max(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_max(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_max_epu16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vmaxq_u16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_max(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_max(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_max_epu32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vmaxq_u32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u32x4_max(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i8x16_max(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_max_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vmaxq_s8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i8x16_max(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i16x8_max(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_max_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vmaxq_s16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i16x8_max(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i32x4_max(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_max_epi32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vmaxq_s32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i32x4_max(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_min(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_min_epu8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vminq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_min(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_min(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_min_epu16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vminq_u16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u16x8_min(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_min(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_min_epu32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vminq_u32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u32x4_min(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i8x16_min(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_min_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vminq_s8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i8x16_min(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i16x8_min(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_min_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vminq_s16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i16x8_min(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i32x4_min(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_min_epi32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vminq_s32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i32x4_min(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_swizzle(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_shuffle_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe {
                let (a, b) = (t(a), t(b));
                let a = uint8x8x2_t(vget_low_u8(a), vget_high_u8(a));
                let b = (vget_low_u8(b), vget_high_u8(b));
                let c = (vtbl2_u8(a, b.0), vtbl2_u8(a, b.1));
                t([c.0, c.1])
            };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vqtbl1q_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_swizzle(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_bswap(self, a: V128) -> V128 {
        if is_subtype!(Self, SSE41 | WASM128) {
            return self.u8x16_swizzle(a, crate::bswap::SHUFFLE_U16X8);
        }

        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vrev16q_u8(t(a))) };
        }

        {
            let _ = a;
            unreachable!()
        }
    }
    #[inline(always)]
    fn u32x4_bswap(self, a: V128) -> V128 {
        if is_subtype!(Self, SSE41 | WASM128) {
            return self.u8x16_swizzle(a, crate::bswap::SHUFFLE_U32X4);
        }

        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vrev32q_u8(t(a))) };
        }

        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u64x2_bswap(self, a: V128) -> V128 {
        if is_subtype!(Self, SSE41 | WASM128) {
            return self.u8x16_swizzle(a, crate::bswap::SHUFFLE_U64X2);
        }

        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vrev64q_u8(t(a))) };
        }

        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_any_zero(self, a: V128) -> bool {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            let is_zero = self.u8x16_eq(a, self.v128_create_zero());
            return self.u8x16_bitmask(is_zero) != 0;
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            let is_zero = self.u8x16_eq(a, self.v128_create_zero());
            return !self.v128_all_zero(is_zero);
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { vminvq_u8(t(a)) == 0 };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { !u8x16_all_true(t(a)) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_bitmask(self, a: V128) -> u16 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { _mm_movemask_epi8(t(a)) as u16 };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            unimplemented!()
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { u8x16_bitmask(t(a)) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_reduce_max(self, a: V128) -> u8 {
        if is_subtype!(Self, SSE41 | WASM128) {
            unimplemented!()
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            unimplemented!()
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { vmaxvq_u8(t(a)) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_reduce_min(self, a: V128) -> u8 {
        if is_subtype!(Self, SSE41 | WASM128) {
            unimplemented!()
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            unimplemented!()
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { vminvq_u8(t(a)) };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn v128_bsl(self, a: V128, b: V128, c: V128) -> V128 {
        if is_subtype!(Self, SSE41 | WASM128) {
            return self.v128_xor(self.v128_and(self.v128_xor(b, c), a), c);
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vbslq_u8(t(a), t(b), t(c))) };
        }
        {
            let _ = (a, b, c);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_zip_lo(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_unpacklo_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzipq_u8(t(a), t(b)).0) };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzip1q_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_zip_hi(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_unpackhi_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzipq_u8(t(a), t(b)).1) };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzip2q_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u8x16_shuffle::<8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_zip_lo(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_unpacklo_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzipq_u16(t(a), t(b)).0) };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzip1q_u16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u16x8_shuffle::<0, 8, 1, 9, 2, 10, 3, 11>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_zip_hi(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_unpackhi_epi16(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzipq_u16(t(a), t(b)).1) };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzip2q_u16(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u16x8_shuffle::<4, 12, 5, 13, 6, 14, 7, 15>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_zip_lo(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_unpacklo_epi32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzipq_u32(t(a), t(b)).0) };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzip1q_u32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u32x4_shuffle::<0, 4, 1, 5>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_zip_hi(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_unpackhi_epi32(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzipq_u32(t(a), t(b)).1) };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vzip2q_u32(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u32x4_shuffle::<2, 6, 3, 7>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u64x2_zip_lo(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_unpacklo_epi64(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe {
                let (a, b): ([u64; 2], [u64; 2]) = (t(a), t(b));
                t([a[0], b[0]])
            };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u64x2_shuffle::<0, 2>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u64x2_zip_hi(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_unpackhi_epi64(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe {
                let (a, b): ([u64; 2], [u64; 2]) = (t(a), t(b));
                t([a[1], b[1]])
            };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u64x2_shuffle::<1, 3>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_unzip_even(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            unimplemented!()
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vuzpq_u8(t(a), t(b)).0) };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vuzp1q_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u8x16_shuffle::<0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_unzip_odd(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            unimplemented!()
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vuzpq_u8(t(a), t(b)).1) };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vuzp2q_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            let (a, b) = unsafe { (t(a), t(b)) };
            let ans = u8x16_shuffle::<1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31>(a, b);
            return unsafe { t(ans) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u16x8_mul_hi(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_mulhi_epu16(t(a), t(b))) };
        }
        if is_subtype!(Self, NEON | WASM128) {
            unimplemented!()
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i16x8_mul_hi(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_mulhi_epi16(t(a), t(b))) };
        }
        if is_subtype!(Self, NEON | WASM128) {
            unimplemented!()
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i16x8_maddubs(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_maddubs_epi16(t(a), t(b))) };
        }
        if is_subtype!(Self, NEON | WASM128) {
            unimplemented!()
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u32x4_blend<const IMM4: i32>(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_blend_epi32::<IMM4>(t(a), t(b))) };
        }
        if is_subtype!(Self, NEON | WASM128) {
            unimplemented!()
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    /// if highbit(c) { b } else { a }
    #[inline(always)]
    fn u8x16_blendv(self, a: V128, b: V128, c: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_blendv_epi8(t(a), t(b), t(c))) };
        }
        if is_subtype!(Self, NEON | WASM128) {
            unimplemented!()
        }
        {
            let _ = (a, b, c);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i16x8_madd(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_madd_epi16(t(a), t(b))) };
        }
        if is_subtype!(Self, NEON | WASM128) {
            unimplemented!()
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_avgr(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_avg_epu8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vrhaddq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_avgr(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn i8x16_add_sat(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_adds_epi8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vqaddq_s8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(i8x16_add_sat(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16_add_sat(self, a: V128, b: V128) -> V128 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE41) {
            return unsafe { t(_mm_adds_epu8(t(a), t(b))) };
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vqaddq_u8(t(a), t(b))) };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe { t(u8x16_add_sat(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }
}
