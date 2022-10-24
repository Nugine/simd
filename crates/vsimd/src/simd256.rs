use crate::isa::{AVX2, NEON, SSE2, WASM128};
use crate::vector::{V128, V256};
use crate::SIMD128;

#[cfg(any(
    any(target_arch = "x86", target_arch = "x86_64"),
    all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")),
    target_arch = "wasm32"
))]
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

#[macro_export]
macro_rules! simd256_vop {
    ($s:expr, $f:expr, $a:expr) => {{
        let s = $s;
        let f = $f;
        let a = $a.to_v128x2();
        let b = (f(s, a.0), f(s, a.1));
        V256::from_v128x2(b)
    }};
    ($s:expr, $f:expr, $a:expr, $b:expr) => {{
        let s = $s;
        let f = $f;
        let a = $a.to_v128x2();
        let b = $b.to_v128x2();
        let c = (f(s, a.0, b.0), f(s, a.1, b.1));
        V256::from_v128x2(c)
    }};
    ($s:expr, $f:expr, $a:expr, $b:expr, $c:expr) => {{
        let s = $s;
        let f = $f;
        let a = $a.to_v128x2();
        let b = $b.to_v128x2();
        let c = $c.to_v128x2();
        let d = (f(s, a.0, b.0, c.0), f(s, a.1, b.1, c.1));
        V256::from_v128x2(d)
    }};
}

pub unsafe trait SIMD256: SIMD128 {
    #[inline(always)]
    unsafe fn v256_load(self, addr: *const u8) -> V256 {
        debug_assert_ptr_align!(addr, 32);

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return t(_mm256_load_si256(addr.cast()));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return t(vld1q_u8_x2(addr.cast()));
        }
        {
            let x0 = self.v128_load(addr);
            let x1 = self.v128_load(addr.add(16));
            V256::from_v128x2((x0, x1))
        }
    }

    #[inline(always)]
    unsafe fn v256_load_unaligned(self, addr: *const u8) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return t(_mm256_loadu_si256(addr.cast()));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return t(vld1q_u8_x2(addr.cast()));
        }
        {
            let x0 = self.v128_load_unaligned(addr);
            let x1 = self.v128_load_unaligned(addr.add(16));
            V256::from_v128x2((x0, x1))
        }
    }

    #[inline(always)]
    unsafe fn v256_store(self, addr: *mut u8, a: V256) {
        debug_assert_ptr_align!(addr, 32);

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return _mm256_store_si256(addr.cast(), t(a));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return vst1q_u8_x2(addr.cast(), t(a));
        }
        {
            let a = a.to_v128x2();
            self.v128_store(addr, a.0);
            self.v128_store(addr.add(16), a.1);
        }
    }

    #[inline(always)]
    unsafe fn v256_store_unaligned(self, addr: *mut u8, a: V256) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return _mm256_storeu_si256(addr.cast(), t(a));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return vst1q_u8_x2(addr.cast(), t(a));
        }
        {
            let a = a.to_v128x2();
            self.v128_store_unaligned(addr, a.0);
            self.v128_store_unaligned(addr.add(16), a.1);
        }
    }

    #[inline(always)]
    fn v256_create_zero(self) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_setzero_si256()) };
        }
        {
            self.v128_create_zero().x2()
        }
    }

    #[inline(always)]
    fn v256_not(self, a: V256) -> V256 {
        if is_subtype!(Self, AVX2) {
            return self.v256_xor(a, self.u8x32_eq(a, a));
        }
        {
            simd256_vop!(self, Self::v128_not, a)
        }
    }

    #[inline(always)]
    fn v256_and(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_and_si256(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::v128_and, a, b)
        }
    }

    #[inline(always)]
    fn v256_or(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_or_si256(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::v128_or, a, b)
        }
    }

    #[inline(always)]
    fn v256_xor(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_xor_si256(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::v128_xor, a, b)
        }
    }

    #[inline(always)]
    fn v256_andnot(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_andnot_si256(t(b), t(a))) };
        }
        {
            simd256_vop!(self, Self::v128_andnot, a, b)
        }
    }

    #[inline(always)]
    fn v256_all_zero(self, a: V256) -> bool {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe {
                let a = t(a);
                _mm256_testz_si256(a, a) != 0
            };
        }
        {
            let a = a.to_v128x2();
            self.v128_all_zero(self.v128_or(a.0, a.1))
        }
    }

    #[inline(always)]
    fn u8x32_splat(self, x: u8) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_set1_epi8(x as i8)) };
        }
        {
            self.u8x16_splat(x).x2()
        }
    }

    #[inline(always)]
    fn u16x16_splat(self, x: u16) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_set1_epi16(x as i16)) };
        }
        {
            self.u16x8_splat(x).x2()
        }
    }

    #[inline(always)]
    fn u32x8_splat(self, x: u32) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_set1_epi32(x as i32)) };
        }
        {
            self.u32x4_splat(x).x2()
        }
    }

    #[inline(always)]
    fn u64x4_splat(self, x: u64) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_set1_epi64x(x as i64)) };
        }
        {
            self.u64x2_splat(x).x2()
        }
    }

    #[inline(always)]
    fn i8x32_splat(self, x: i8) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_set1_epi8(x)) };
        }
        {
            self.i8x16_splat(x).x2()
        }
    }

    #[inline(always)]
    fn i16x16_splat(self, x: i16) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_set1_epi16(x)) };
        }
        {
            self.i16x8_splat(x).x2()
        }
    }

    #[inline(always)]
    fn i32x8_splat(self, x: i32) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_set1_epi32(x)) };
        }
        {
            self.i32x4_splat(x).x2()
        }
    }

    #[inline(always)]
    fn i64x4_splat(self, x: i64) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_set1_epi64x(x)) };
        }
        {
            self.i64x2_splat(x).x2()
        }
    }

    #[inline(always)]
    fn u8x32_add(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_add_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_add, a, b)
        }
    }

    #[inline(always)]
    fn u16x16_add(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_add_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u16x8_add, a, b)
        }
    }

    #[inline(always)]
    fn u32x8_add(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_add_epi32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u32x4_add, a, b)
        }
    }

    #[inline(always)]
    fn u64x4_add(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_add_epi64(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u64x2_add, a, b)
        }
    }

    #[inline(always)]
    fn u8x32_sub(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_sub_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_sub, a, b)
        }
    }

    #[inline(always)]
    fn u16x16_sub(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_sub_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u16x8_sub, a, b)
        }
    }

    #[inline(always)]
    fn u32x8_sub(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_sub_epi32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u32x4_sub, a, b)
        }
    }

    #[inline(always)]
    fn u64x4_sub(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_sub_epi64(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u64x2_sub, a, b)
        }
    }

    #[inline(always)]
    fn u8x32_sub_sat(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_subs_epu8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_sub_sat, a, b)
        }
    }

    #[inline(always)]
    fn u16x16_sub_sat(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_subs_epu16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u16x8_sub_sat, a, b)
        }
    }

    #[inline(always)]
    fn i8x32_sub_sat(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_subs_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i8x16_sub_sat, a, b)
        }
    }

    #[inline(always)]
    fn i16x16_sub_sat(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_subs_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i16x8_sub_sat, a, b)
        }
    }

    #[inline(always)]
    fn i16x16_mul_lo(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_mullo_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i16x8_mul_lo, a, b)
        }
    }

    #[inline(always)]
    fn i32x8_mul_lo(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_mullo_epi32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i32x4_mul_lo, a, b)
        }
    }

    #[inline(always)]
    fn u16x16_shl<const IMM8: i32>(self, a: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_slli_epi16::<IMM8>(t(a))) };
        }
        {
            simd256_vop!(self, Self::u16x8_shl::<IMM8>, a)
        }
    }

    #[inline(always)]
    fn u32x8_shl<const IMM8: i32>(self, a: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_slli_epi32::<IMM8>(t(a))) };
        }
        {
            simd256_vop!(self, Self::u32x4_shl::<IMM8>, a)
        }
    }

    #[inline(always)]
    fn u16x16_shr<const IMM8: i32>(self, a: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_srli_epi16::<IMM8>(t(a))) };
        }
        {
            simd256_vop!(self, Self::u16x8_shr::<IMM8>, a)
        }
    }

    #[inline(always)]
    fn u32x8_shr<const IMM8: i32>(self, a: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_srli_epi32::<IMM8>(t(a))) };
        }
        {
            simd256_vop!(self, Self::u32x4_shr::<IMM8>, a)
        }
    }

    #[inline(always)]
    fn u8x32_eq(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_cmpeq_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_eq, a, b)
        }
    }

    #[inline(always)]
    fn u16x16_eq(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_cmpeq_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u16x8_eq, a, b)
        }
    }

    #[inline(always)]
    fn u32x8_eq(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_cmpeq_epi32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u32x4_eq, a, b)
        }
    }

    #[inline(always)]
    fn u8x32_lt(self, a: V256, b: V256) -> V256 {
        if is_subtype!(Self, AVX2) {
            let shift = self.u8x32_splat(u8::MAX / 2);
            let a = self.u8x32_sub(a, shift);
            let b = self.u8x32_sub(b, shift);
            return self.i8x32_lt(a, b);
        }
        {
            simd256_vop!(self, Self::u8x16_lt, a, b)
        }
    }

    #[inline(always)]
    fn u16x16_lt(self, a: V256, b: V256) -> V256 {
        if is_subtype!(Self, AVX2) {
            let shift = self.u16x16_splat(u16::MAX / 2);
            let a = self.u16x16_sub(a, shift);
            let b = self.u16x16_sub(b, shift);
            return self.i16x16_lt(a, b);
        }
        {
            simd256_vop!(self, Self::u16x8_lt, a, b)
        }
    }

    #[inline(always)]
    fn u32x8_lt(self, a: V256, b: V256) -> V256 {
        if is_subtype!(Self, AVX2) {
            let shift = self.u32x8_splat(u32::MAX / 2);
            let a = self.u32x8_sub(a, shift);
            let b = self.u32x8_sub(b, shift);
            return self.i32x8_lt(a, b);
        }
        {
            simd256_vop!(self, Self::u32x4_lt, a, b)
        }
    }

    #[inline(always)]
    fn i8x32_lt(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_cmpgt_epi8(t(b), t(a))) };
        }
        {
            simd256_vop!(self, Self::i8x16_lt, a, b)
        }
    }

    #[inline(always)]
    fn i16x16_lt(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_cmpgt_epi16(t(b), t(a))) };
        }
        {
            simd256_vop!(self, Self::i16x8_lt, a, b)
        }
    }

    #[inline(always)]
    fn i32x8_lt(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_cmpgt_epi32(t(b), t(a))) };
        }
        {
            simd256_vop!(self, Self::i32x4_lt, a, b)
        }
    }

    #[inline(always)]
    fn u8x32_max(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_max_epu8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_max, a, b)
        }
    }

    #[inline(always)]
    fn u16x16_max(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_max_epu16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u16x8_max, a, b)
        }
    }

    #[inline(always)]
    fn u32x8_max(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_max_epu32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u32x4_max, a, b)
        }
    }

    #[inline(always)]
    fn i8x32_max(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_max_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i8x16_max, a, b)
        }
    }

    #[inline(always)]
    fn i16x16_max(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_max_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i16x8_max, a, b)
        }
    }

    #[inline(always)]
    fn i32x8_max(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_max_epi32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i32x4_max, a, b)
        }
    }

    #[inline(always)]
    fn u8x32_min(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_min_epu8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_min, a, b)
        }
    }

    #[inline(always)]
    fn u16x16_min(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_min_epu16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u16x8_min, a, b)
        }
    }

    #[inline(always)]
    fn u32x8_min(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_min_epu32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u32x4_min, a, b)
        }
    }

    #[inline(always)]
    fn i8x32_min(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_min_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i8x16_min, a, b)
        }
    }

    #[inline(always)]
    fn i16x16_min(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_min_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i16x8_min, a, b)
        }
    }

    #[inline(always)]
    fn i32x8_min(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_min_epi32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i32x4_min, a, b)
        }
    }

    #[inline(always)]
    fn u8x16x2_swizzle(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_shuffle_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_swizzle, a, b)
        }
    }

    #[inline(always)]
    fn u16x16_bswap(self, a: V256) -> V256 {
        if is_subtype!(Self, AVX2) {
            return self.u8x16x2_swizzle(a, crate::bswap::SHUFFLE_U16X16);
        }
        {
            simd256_vop!(self, Self::u16x8_bswap, a)
        }
    }

    #[inline(always)]
    fn u32x8_bswap(self, a: V256) -> V256 {
        if is_subtype!(Self, AVX2) {
            return self.u8x16x2_swizzle(a, crate::bswap::SHUFFLE_U32X8);
        }
        {
            simd256_vop!(self, Self::u32x4_bswap, a)
        }
    }

    #[inline(always)]
    fn u64x4_bswap(self, a: V256) -> V256 {
        if is_subtype!(Self, AVX2) {
            return self.u8x16x2_swizzle(a, crate::bswap::SHUFFLE_U64X4);
        }
        {
            simd256_vop!(self, Self::u64x2_bswap, a)
        }
    }

    #[inline(always)]
    fn u8x32_swizzle(self, a: V256, b: V256) -> V256 {
        if is_subtype!(Self, SSE2 | WASM128) {
            let _ = (a, b);
            unimplemented!()
        }
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            let _ = (a, b);
            unimplemented!()
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe {
                let (a, b): (uint8x16x2_t, uint8x16x2_t) = (t(a), t(b));
                let c = (vqtbl2q_u8(a, b.0), vqtbl2q_u8(a, b.1));
                t(uint8x16x2_t(c.0, c.1))
            };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x32_any_zero(self, a: V256) -> bool {
        if is_subtype!(Self, AVX2) {
            let is_zero = self.u8x32_eq(a, self.v256_create_zero());
            return self.u8x32_bitmask(is_zero) != 0;
        }
        {
            let a = a.to_v128x2();
            self.u8x16_any_zero(self.u8x16_min(a.0, a.1))
        }
    }

    #[inline(always)]
    fn u8x32_bitmask(self, a: V256) -> u32 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { _mm256_movemask_epi8(t(a)) as u32 };
        }
        {
            let a = a.to_v128x2();
            let m0 = self.u8x16_bitmask(a.0) as u32;
            let m1 = self.u8x16_bitmask(a.1) as u32;
            (m1 << 16) | m0
        }
    }

    #[inline(always)]
    fn u8x32_reduce_max(self, a: V256) -> u8 {
        let a = a.to_v128x2();
        self.u8x16_reduce_max(self.u8x16_max(a.0, a.1))
    }

    #[inline(always)]
    fn u8x32_reduce_min(self, a: V256) -> u8 {
        let a = a.to_v128x2();
        self.u8x16_reduce_min(self.u8x16_min(a.0, a.1))
    }

    /// for each bit: if a == 1 { b } else { c }
    ///
    /// ans = ((b ^ c) & a) ^ c
    #[inline(always)]
    fn v256_bsl(self, a: V256, b: V256, c: V256) -> V256 {
        if is_subtype!(Self, NEON) {
            return simd256_vop!(self, Self::v128_bsl, a, b, c);
        }
        {
            self.v256_xor(self.v256_and(self.v256_xor(b, c), a), c)
        }
    }

    #[inline(always)]
    fn u16x16_from_u8x16(self, a: V128) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_cvtepu8_epi16(t(a))) };
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, SSE2) {
            let zero = self.v128_create_zero();
            let lo = self.u8x16_zip_lo(a, zero);
            let hi = self.u8x16_zip_hi(a, zero);
            return V256::from_v128x2((lo, hi));
        }
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if is_subtype!(Self, NEON) {
            return unsafe {
                let a = t(a);
                let low = vmovl_u8(vget_low_u8(a));
                let high = vmovl_u8(vget_high_u8(a));
                t(uint16x8x2_t(low, high))
            };
        }
        #[cfg(target_arch = "wasm32")]
        if is_subtype!(Self, WASM128) {
            return unsafe {
                let a = t(a);
                let low = t(u16x8_extend_low_u8x16(a));
                let high = t(u16x8_extend_high_u8x16(a));
                V256::from_v128x2((low, high))
            };
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x16x2_zip_lo(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_unpacklo_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_zip_lo, a, b)
        }
    }

    #[inline(always)]
    fn u8x16x2_zip_hi(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_unpackhi_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_zip_hi, a, b)
        }
    }

    #[inline(always)]
    fn u16x8x2_zip_lo(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_unpacklo_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u16x8_zip_lo, a, b)
        }
    }

    #[inline(always)]
    fn u16x8x2_zip_hi(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_unpackhi_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u16x8_zip_hi, a, b)
        }
    }

    #[inline(always)]
    fn u32x4x2_zip_lo(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_unpacklo_epi32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u32x4_zip_lo, a, b)
        }
    }

    #[inline(always)]
    fn u32x4x2_zip_hi(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_unpackhi_epi32(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u32x4_zip_hi, a, b)
        }
    }

    #[inline(always)]
    fn u64x2x2_zip_lo(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_unpacklo_epi64(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u64x2_zip_lo, a, b)
        }
    }

    #[inline(always)]
    fn u64x2x2_zip_hi(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_unpackhi_epi64(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u64x2_zip_hi, a, b)
        }
    }

    #[inline(always)]
    fn v128x2_zip_lo(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_permute2x128_si256::<0b0010_0000>(t(a), t(b))) };
        }
        if is_subtype!(Self, SSE2 | NEON | WASM128) {
            let ((a, _), (c, _)) = (a.to_v128x2(), b.to_v128x2());
            return V256::from_v128x2((a, c));
        }
        {
            unreachable!()
        }
    }

    #[inline(always)]
    fn v128x2_zip_hi(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_permute2x128_si256::<0b0011_0001>(t(a), t(b))) };
        }
        if is_subtype!(Self, SSE2 | NEON | WASM128) {
            let ((_, b), (_, d)) = (a.to_v128x2(), b.to_v128x2());
            return V256::from_v128x2((b, d));
        }
        {
            unreachable!()
        }
    }

    #[inline(always)]
    fn u64x4_permute<const IMM8: i32>(self, a: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_permute4x64_epi64::<IMM8>(t(a))) };
        }
        if is_subtype!(Self, SSE2 | NEON | WASM128) {
            let _ = a;
            unimplemented!()
        }
        {
            let _ = a;
            unreachable!()
        }
    }

    #[inline(always)]
    fn u8x32_unzip_even(self, a: V256, b: V256) -> V256 {
        if is_subtype!(Self, SSE2) {
            unimplemented!()
        }
        {
            let ((a, b), (c, d)) = (a.to_v128x2(), b.to_v128x2());
            let ab = self.u8x16_unzip_even(a, b);
            let cd = self.u8x16_unzip_even(c, d);
            V256::from_v128x2((ab, cd))
        }
    }

    #[inline(always)]
    fn u8x32_unzip_odd(self, a: V256, b: V256) -> V256 {
        if is_subtype!(Self, SSE2) {
            unimplemented!()
        }
        {
            let ((a, b), (c, d)) = (a.to_v128x2(), b.to_v128x2());
            let ab = self.u8x16_unzip_odd(a, b);
            let cd = self.u8x16_unzip_odd(c, d);
            V256::from_v128x2((ab, cd))
        }
    }

    #[inline(always)]
    fn u64x4_unzip_even(self, a: V256, b: V256) -> V256 {
        if is_subtype!(Self, AVX2) {
            let acbd = self.u64x2x2_zip_lo(a, b);
            let abcd = self.u64x4_permute::<0b_1101_1000>(acbd); // 0213
            return abcd;
        }
        {
            let ((a, b), (c, d)) = (a.to_v128x2(), b.to_v128x2());
            let ab = self.u64x2_zip_lo(a, b);
            let cd = self.u64x2_zip_lo(c, d);
            V256::from_v128x2((ab, cd))
        }
    }

    #[inline(always)]
    fn u64x4_unzip_odd(self, a: V256, b: V256) -> V256 {
        if is_subtype!(Self, AVX2) {
            let acbd = self.u64x2x2_zip_hi(a, b);
            let abcd = self.u64x4_permute::<0b_1101_1000>(acbd); // 0213
            return abcd;
        }
        {
            let ((a, b), (c, d)) = (a.to_v128x2(), b.to_v128x2());
            let ab = self.u64x2_zip_hi(a, b);
            let cd = self.u64x2_zip_hi(c, d);
            V256::from_v128x2((ab, cd))
        }
    }

    #[inline(always)]
    fn u16x16_mul_hi(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_mulhi_epu16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u16x8_mul_hi, a, b)
        }
    }

    #[inline(always)]
    fn i16x16_mul_hi(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_mulhi_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i16x8_mul_hi, a, b)
        }
    }

    #[inline(always)]
    fn i16x16_maddubs(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_maddubs_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i16x8_maddubs, a, b)
        }
    }

    #[inline(always)]
    fn u32x8_blend<const IMM8: i32>(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_blend_epi32::<IMM8>(t(a), t(b))) };
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
    fn u8x32_blendv(self, a: V256, b: V256, c: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_blendv_epi8(t(a), t(b), t(c))) };
        }
        if is_subtype!(Self, NEON | WASM128) {
            unimplemented!()
        }
        {
            simd256_vop!(self, Self::u8x16_blendv, a, b, c)
        }
    }

    #[inline(always)]
    fn i16x16_madd(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_madd_epi16(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i16x8_madd, a, b)
        }
    }

    #[inline(always)]
    fn u8x32_avgr(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_avg_epu8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_avgr, a, b)
        }
    }

    #[inline(always)]
    fn i8x32_add_sat(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_adds_epi8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::i8x16_add_sat, a, b)
        }
    }

    #[inline(always)]
    fn u8x32_add_sat(self, a: V256, b: V256) -> V256 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_subtype!(Self, AVX2) {
            return unsafe { t(_mm256_adds_epu8(t(a), t(b))) };
        }
        {
            simd256_vop!(self, Self::u8x16_add_sat, a, b)
        }
    }
}
