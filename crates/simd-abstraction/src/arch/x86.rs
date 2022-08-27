use crate::isa::mock::*;
use crate::isa::SimdLoad;
use crate::isa::{InstructionSet, SIMD128, SIMD256, SIMD512};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

define_isa!(SSE41, "sse4.1");
define_isa!(SSE42, "sse4.2");
define_isa!(AVX2, "avx2");

unsafe impl SIMD128 for SSE41 {
    type V128 = __m128i;

    #[inline(always)]
    fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
        unsafe { core::mem::transmute(a) }
    }

    #[inline(always)]
    unsafe fn v128_load(self, addr: *const u8) -> Self::V128 {
        debug_assert_ptr_align!(addr, 16);
        _mm_load_si128(addr.cast()) // sse2
    }

    #[inline(always)]
    unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128 {
        _mm_loadu_si128(addr.cast()) // sse2
    }

    #[inline(always)]
    unsafe fn v128_store(self, addr: *mut u8, a: Self::V128) {
        debug_assert_ptr_align!(addr, 16);
        _mm_store_si128(addr.cast(), a) // sse2
    }

    #[inline(always)]
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
        _mm_storeu_si128(addr.cast(), a) // sse2
    }

    #[inline(always)]
    fn v128_create_zero(self) -> Self::V128 {
        unsafe { _mm_setzero_si128() } // sse2
    }

    #[inline(always)]
    fn v128_not(self, a: Self::V128) -> Self::V128 {
        self.v128_xor(a, self.u8x16_eq(a, a))
    }

    #[inline(always)]
    fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_and_si128(a, b) } // sse2
    }

    #[inline(always)]
    fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_or_si128(a, b) } // sse2
    }

    #[inline(always)]
    fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_xor_si128(a, b) } // sse2
    }

    #[inline(always)]
    fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_andnot_si128(b, a) } // sse2
    }

    #[inline(always)]
    fn v128_all_zero(self, a: Self::V128) -> bool {
        unsafe { _mm_testz_si128(a, a) != 0 } // sse41
    }

    #[inline(always)]
    fn u8x16_splat(self, x: u8) -> Self::V128 {
        unsafe { _mm_set1_epi8(x as i8) } // sse2
    }

    #[inline(always)]
    fn u16x8_splat(self, x: u16) -> Self::V128 {
        unsafe { _mm_set1_epi16(x as i16) } // sse2
    }

    #[inline(always)]
    fn u32x4_splat(self, x: u32) -> Self::V128 {
        unsafe { _mm_set1_epi32(x as i32) } // sse2
    }

    #[inline(always)]
    fn u64x2_splat(self, x: u64) -> Self::V128 {
        unsafe { _mm_set1_epi64x(x as i64) } // sse2
    }

    #[inline(always)]
    fn i8x16_splat(self, x: i8) -> Self::V128 {
        unsafe { _mm_set1_epi8(x) } // sse2
    }

    #[inline(always)]
    fn i16x8_splat(self, x: i16) -> Self::V128 {
        unsafe { _mm_set1_epi16(x) } // sse2
    }

    #[inline(always)]
    fn i32x4_splat(self, x: i32) -> Self::V128 {
        unsafe { _mm_set1_epi32(x) } // sse2
    }

    #[inline(always)]
    fn i64x2_splat(self, x: i64) -> Self::V128 {
        unsafe { _mm_set1_epi64x(x) } // sse2
    }

    #[inline(always)]
    fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_add_epi8(a, b) } // sse2
    }

    #[inline(always)]
    fn u16x8_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_add_epi16(a, b) } // sse2
    }

    #[inline(always)]
    fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_add_epi32(a, b) } // sse2
    }

    #[inline(always)]
    fn u64x2_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_add_epi64(a, b) } // sse2
    }

    #[inline(always)]
    fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_sub_epi8(a, b) } // sse2
    }

    #[inline(always)]
    fn u16x8_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_sub_epi16(a, b) } // sse2
    }

    #[inline(always)]
    fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_sub_epi32(a, b) } // sse2
    }

    #[inline(always)]
    fn u64x2_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_sub_epi64(a, b) } // sse2
    }

    #[inline(always)]
    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_subs_epu8(a, b) } // sse2
    }

    #[inline(always)]
    fn u16x8_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_subs_epu16(a, b) } // sse2
    }

    #[inline(always)]
    fn i8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_subs_epi8(a, b) } // sse2
    }

    #[inline(always)]
    fn i16x8_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_subs_epi16(a, b) } // sse2
    }

    #[inline(always)]
    fn i16x8_mul_lo(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_mullo_epi16(a, b) } // sse2
    }

    #[inline(always)]
    fn i32x4_mul_lo(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_mullo_epi32(a, b) } // sse2
    }

    #[inline(always)]
    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { _mm_slli_epi16::<IMM8>(a) } // sse2
    }

    #[inline(always)]
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { _mm_slli_epi32::<IMM8>(a) } // sse2
    }

    #[inline(always)]
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { _mm_srli_epi16::<IMM8>(a) } // sse2
    }

    #[inline(always)]
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { _mm_srli_epi32::<IMM8>(a) } // sse2
    }

    #[inline(always)]
    fn u8x16_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_cmpeq_epi8(a, b) } // sse2
    }

    #[inline(always)]
    fn u16x8_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_cmpeq_epi16(a, b) } // sse2
    }

    #[inline(always)]
    fn u32x4_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_cmpeq_epi32(a, b) } // sse2
    }

    #[inline(always)]
    fn u8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        let shift = self.u8x16_splat(u8::MAX / 2);
        let a = self.u8x16_sub(a, shift);
        let b = self.u8x16_sub(b, shift);
        self.i8x16_lt(a, b)
    }

    #[inline(always)]
    fn u16x8_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        let shift = self.u16x8_splat(u16::MAX / 2);
        let a = self.u16x8_sub(a, b);
        let b = self.u16x8_sub(b, shift);
        self.i16x8_lt(a, b)
    }

    #[inline(always)]
    fn u32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        let shift = self.u32x4_splat(u32::MAX / 2);
        let a = self.u32x4_sub(a, shift);
        let b = self.u32x4_sub(b, shift);
        self.i32x4_lt(a, b)
    }

    #[inline(always)]
    fn i8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_cmplt_epi8(a, b) } // sse2
    }

    #[inline(always)]
    fn i16x8_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_cmplt_epi16(a, b) } // sse2
    }

    #[inline(always)]
    fn i32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_cmplt_epi32(a, b) } // sse2
    }

    #[inline(always)]
    fn u8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_max_epu8(a, b) } // sse2
    }

    #[inline(always)]
    fn u16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_max_epu16(a, b) } // sse41
    }

    #[inline(always)]
    fn u32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_max_epu32(a, b) } // sse41
    }

    #[inline(always)]
    fn i8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_max_epi8(a, b) } // sse41
    }

    #[inline(always)]
    fn i16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_max_epi16(a, b) } // sse2
    }

    #[inline(always)]
    fn i32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_max_epi32(a, b) } // sse41
    }

    #[inline(always)]
    fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_min_epu8(a, b) } // sse2
    }

    #[inline(always)]
    fn u16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_min_epu16(a, b) } // sse41
    }

    #[inline(always)]
    fn u32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_min_epu32(a, b) } // sse41
    }

    #[inline(always)]
    fn i8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_min_epi8(a, b) } // sse41
    }

    #[inline(always)]
    fn i16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_min_epi16(a, b) } // sse2
    }

    #[inline(always)]
    fn i32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_min_epi32(a, b) } // sse41
    }

    #[inline(always)]
    fn u16x8_bswap(self, a: Self::V128) -> Self::V128 {
        self.u8x16_swizzle(a, self.load(crate::common::bswap::SHUFFLE_U16X8))
    }

    #[inline(always)]
    fn u32x4_bswap(self, a: Self::V128) -> Self::V128 {
        self.u8x16_swizzle(a, self.load(crate::common::bswap::SHUFFLE_U32X4))
    }

    #[inline(always)]
    fn u64x2_bswap(self, a: Self::V128) -> Self::V128 {
        self.u8x16_swizzle(a, self.load(crate::common::bswap::SHUFFLE_U64X2))
    }

    #[inline(always)]
    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_shuffle_epi8(a, b) } // ssse3
    }

    #[inline(always)]
    fn u8x16_any_zero(self, a: Self::V128) -> bool {
        let zero = self.v128_create_zero();
        let cmp = self.u8x16_eq(a, zero);
        unsafe { _mm_movemask_epi8(cmp) != 0 }
    }
}

unsafe impl SIMD256 for SSE41 {
    type V256 = (__m128i, __m128i);

    #[inline(always)]
    fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256 {
        (a, b)
    }

    #[inline(always)]
    fn v256_to_v128x2(self, a: Self::V256) -> (Self::V128, Self::V128) {
        (a.0, a.1)
    }

    #[inline(always)]
    fn v256_to_bytes(self, a: Self::V256) -> [u8; 32] {
        unsafe { core::mem::transmute([a.0, a.1]) }
    }
}

unsafe impl SIMD512 for SSE41 {
    type V512 = (__m128i, __m128i, __m128i, __m128i);

    #[inline(always)]
    fn v512_from_v256x2(self, a: Self::V256, b: Self::V256) -> Self::V512 {
        (a.0, a.1, b.0, b.1)
    }

    #[inline(always)]
    fn v512_to_v256x2(self, a: Self::V512) -> (Self::V256, Self::V256) {
        ((a.0, a.1), (a.2, a.3))
    }

    #[inline(always)]
    fn v512_to_bytes(self, a: Self::V512) -> [u8; 64] {
        unsafe { core::mem::transmute([a.0, a.1, a.2, a.3]) }
    }
}

impl SSE42 {
    #[inline(always)]
    fn sse41(self) -> SSE41 {
        unsafe { SSE41::new() }
    }
}

inherit_simd128!(SSE42, SSE41, sse41);
inherit_simd256!(SSE42, SSE41, sse41);
inherit_simd512!(SSE42, SSE41, sse41);

impl AVX2 {
    #[inline(always)]
    fn sse41(self) -> SSE41 {
        unsafe { SSE41::new() }
    }
}

inherit_simd128!(AVX2, SSE41, sse41);

unsafe impl SIMD256 for AVX2 {
    type V256 = __m256i;

    #[inline(always)]
    fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256 {
        unsafe { _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(a), b) } // avx2
    }

    #[inline(always)]
    fn v256_to_v128x2(self, a: Self::V256) -> (Self::V128, Self::V128) {
        (self.v256_get_low(a), self.v256_get_high(a))
    }

    #[inline(always)]
    fn v256_to_bytes(self, a: Self::V256) -> [u8; 32] {
        unsafe { core::mem::transmute(a) }
    }

    #[inline(always)]
    unsafe fn v256_load(self, addr: *const u8) -> Self::V256 {
        debug_assert_ptr_align!(addr, 32);
        _mm256_load_si256(addr.cast()) // avx
    }

    #[inline(always)]
    unsafe fn v256_load_unaligned(self, addr: *const u8) -> Self::V256 {
        _mm256_loadu_si256(addr.cast()) // avx
    }

    #[inline(always)]
    unsafe fn v256_store(self, addr: *mut u8, a: Self::V256) {
        debug_assert_ptr_align!(addr, 32);
        _mm256_store_si256(addr.cast(), a) // avx
    }

    #[inline(always)]
    unsafe fn v256_store_unaligned(self, addr: *mut u8, a: Self::V256) {
        _mm256_storeu_si256(addr.cast(), a) // avx
    }

    #[inline(always)]
    fn v256_create_zero(self) -> Self::V256 {
        unsafe { _mm256_setzero_si256() } // avx
    }

    #[inline(always)]
    fn v256_get_low(self, a: Self::V256) -> Self::V128 {
        unsafe { _mm256_castsi256_si128(a) } // avx
    }

    #[inline(always)]
    fn v256_get_high(self, a: Self::V256) -> Self::V128 {
        unsafe { _mm256_extracti128_si256::<1>(a) } // avx2
    }

    #[inline(always)]
    fn v256_not(self, a: Self::V256) -> Self::V256 {
        self.v256_xor(a, self.u8x32_eq(a, a))
    }

    #[inline(always)]
    fn v256_and(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_and_si256(a, b) } // avx2
    }

    #[inline(always)]
    fn v256_or(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_or_si256(a, b) } // avx2
    }

    #[inline(always)]
    fn v256_xor(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_xor_si256(a, b) } // avx
    }

    #[inline(always)]
    fn v256_andnot(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_andnot_si256(b, a) } // avx2
    }

    #[inline(always)]
    fn v256_all_zero(self, a: Self::V256) -> bool {
        unsafe { _mm256_testz_si256(a, a) != 0 } // avx
    }

    #[inline(always)]
    fn u8x32_splat(self, x: u8) -> Self::V256 {
        unsafe { _mm256_set1_epi8(x as i8) } // avx
    }

    #[inline(always)]
    fn u16x16_splat(self, x: u16) -> Self::V256 {
        unsafe { _mm256_set1_epi16(x as i16) } // avx
    }

    #[inline(always)]
    fn u32x8_splat(self, x: u32) -> Self::V256 {
        unsafe { _mm256_set1_epi32(x as i32) } // avx
    }

    #[inline(always)]
    fn u64x4_splat(self, x: u64) -> Self::V256 {
        unsafe { _mm256_set1_epi64x(x as i64) } // avx
    }

    #[inline(always)]
    fn i8x32_splat(self, x: i8) -> Self::V256 {
        unsafe { _mm256_set1_epi8(x) } // avx
    }

    #[inline(always)]
    fn i16x16_splat(self, x: i16) -> Self::V256 {
        unsafe { _mm256_set1_epi16(x) } // avx
    }

    #[inline(always)]
    fn i32x8_splat(self, x: i32) -> Self::V256 {
        unsafe { _mm256_set1_epi32(x) } // avx
    }

    #[inline(always)]
    fn i64x4_splat(self, x: i64) -> Self::V256 {
        unsafe { _mm256_set1_epi64x(x) } // avx
    }

    #[inline(always)]
    fn u8x32_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_add_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn u16x16_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_add_epi16(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_add_epi32(a, b) } // avx2
    }

    #[inline(always)]
    fn u64x4_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_add_epi64(a, b) } // avx2
    }

    #[inline(always)]
    fn u8x32_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_sub_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn u16x16_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_sub_epi16(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_sub_epi32(a, b) } // avx2
    }

    #[inline(always)]
    fn u64x4_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_sub_epi64(a, b) } // avx2
    }

    #[inline(always)]
    fn u8x32_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_subs_epu8(a, b) } // avx2
    }

    #[inline(always)]
    fn u16x16_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_subs_epu16(a, b) } // avx2
    }

    #[inline(always)]
    fn i8x32_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_subs_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn i16x16_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_subs_epi16(a, b) } // avx2
    }

    #[inline(always)]
    fn i16x16_mul_lo(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_mullo_epi16(a, b) } // avx2
    }

    #[inline(always)]
    fn i32x8_mul_lo(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_mullo_epi32(a, b) } // avx2
    }

    #[inline(always)]
    fn u16x16_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_slli_epi16::<IMM8>(a) } // avx2
    }

    #[inline(always)]
    fn u32x8_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_slli_epi32::<IMM8>(a) } // avx2
    }

    #[inline(always)]
    fn u16x16_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_srli_epi16::<IMM8>(a) } // avx2
    }

    #[inline(always)]
    fn u32x8_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_srli_epi32::<IMM8>(a) } // avx2
    }

    #[inline(always)]
    fn u8x32_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpeq_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn u16x16_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpeq_epi16(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpeq_epi32(a, b) } // avx2
    }

    #[inline(always)]
    fn u8x32_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        let shift = self.u8x32_splat(u8::MAX / 2);
        let a = self.u8x32_sub(a, shift);
        let b = self.u8x32_sub(b, shift);
        self.i8x32_lt(a, b)
    }

    #[inline(always)]
    fn u16x16_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        let shift = self.u16x16_splat(u16::MAX / 2);
        let a = self.u16x16_sub(a, b);
        let b = self.u16x16_sub(b, shift);
        self.i16x16_lt(a, b)
    }

    #[inline(always)]
    fn u32x8_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        let shift = self.u32x8_splat(u32::MAX / 2);
        let a = self.u32x8_sub(a, shift);
        let b = self.u32x8_sub(b, shift);
        self.i32x8_lt(a, b)
    }

    #[inline(always)]
    fn i8x32_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpgt_epi8(b, a) } // avx2
    }

    #[inline(always)]
    fn i16x16_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpgt_epi16(b, a) } // avx2
    }

    #[inline(always)]
    fn i32x8_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpgt_epi32(b, a) } // avx2
    }

    #[inline(always)]
    fn u8x32_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_max_epu8(a, b) } // avx2
    }

    #[inline(always)]
    fn u16x16_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_max_epu16(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_max_epu32(a, b) } // avx2
    }

    #[inline(always)]
    fn i8x32_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_max_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn i16x16_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_max_epi16(a, b) } // avx2
    }

    #[inline(always)]
    fn i32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_max_epi32(a, b) } // avx2
    }

    #[inline(always)]
    fn u8x32_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_min_epu8(a, b) } // avx2
    }

    #[inline(always)]
    fn u16x16_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_min_epu16(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_min_epu32(a, b) } // avx2
    }

    #[inline(always)]
    fn i8x32_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_min_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn i16x16_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_min_epi16(a, b) } // avx2
    }

    #[inline(always)]
    fn i32x8_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_min_epi32(a, b) } // avx2
    }

    #[inline(always)]
    fn u16x16_bswap(self, a: Self::V256) -> Self::V256 {
        self.u8x16x2_swizzle(a, self.load(crate::common::bswap::SHUFFLE_U16X16))
    }

    #[inline(always)]
    fn u32x8_bswap(self, a: Self::V256) -> Self::V256 {
        self.u8x16x2_swizzle(a, self.load(crate::common::bswap::SHUFFLE_U32X8))
    }

    #[inline(always)]
    fn u64x4_bswap(self, a: Self::V256) -> Self::V256 {
        self.u8x16x2_swizzle(a, self.load(crate::common::bswap::SHUFFLE_U64X4))
    }

    #[inline(always)]
    fn u8x16x2_swizzle(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_shuffle_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn u8x32_any_zero(self, a: Self::V256) -> bool {
        let zero = self.v256_create_zero();
        let cmp = self.u8x32_eq(a, zero);
        unsafe { _mm256_movemask_epi8(cmp) != 0 } // avx2
    }
}

unsafe impl SIMD512 for AVX2 {
    type V512 = (__m256i, __m256i);

    #[inline(always)]
    fn v512_from_v256x2(self, a: Self::V256, b: Self::V256) -> Self::V512 {
        (a, b)
    }

    #[inline(always)]
    fn v512_to_v256x2(self, a: Self::V512) -> (Self::V256, Self::V256) {
        (a.0, a.1)
    }

    #[inline(always)]
    fn v512_to_bytes(self, a: Self::V512) -> [u8; 64] {
        unsafe { core::mem::transmute([a.0, a.1]) }
    }
}

#[inline(always)]
pub fn simd256_vop3<S: SIMD256>(
    s: S,
    a: S::V256,
    b: S::V256,
    c: S::V256,
    f: impl Fn(S, S::V128, S::V128, S::V128) -> S::V128,
) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = s.v256_to_v128x2(b);
    let c = s.v256_to_v128x2(c);
    let d = (f(s, a.0, b.0, c.0), f(s, a.1, b.1, c.1));
    s.v256_from_v128x2(d.0, d.1)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait SIMD128Ext: SIMD128 {
    fn i16x8_mul_hi(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_mul_hi(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u8x16_blendv(self, a: Self::V128, b: Self::V128, m: Self::V128) -> Self::V128;
    fn i16x8_maddubs(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_blend<const IMM8: i32>(self, a: Self::V128, b: Self::V128) -> Self::V128;
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait SIMD256Ext: SIMD256 + SIMD128Ext {
    #[inline(always)]
    fn i16x16_mul_hi(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        simd256_vop2(self, a, b, Self::i16x8_mul_hi)
    }

    #[inline(always)]
    fn u16x16_mul_hi(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        simd256_vop2(self, a, b, Self::u16x8_mul_hi)
    }

    #[inline(always)]
    fn u8x32_blendv(self, a: Self::V256, b: Self::V256, m: Self::V256) -> Self::V256 {
        simd256_vop3(self, a, b, m, Self::u8x16_blendv)
    }

    #[inline(always)]
    fn i16x16_maddubs(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        simd256_vop2(self, a, b, Self::i16x8_maddubs)
    }

    #[inline(always)]
    fn u32x8_blend<const IMM8: i32>(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        simd256_vop2(self, a, b, Self::u32x4_blend::<IMM8>)
    }
}

unsafe impl SIMD128Ext for SSE41 {
    #[inline(always)]
    fn i16x8_mul_hi(self, a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_mulhi_epi16(a, b) } // sse2
    }

    #[inline(always)]
    fn u16x8_mul_hi(self, a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_mulhi_epu16(a, b) } // sse2
    }

    #[inline(always)]
    fn u8x16_blendv(self, a: __m128i, b: __m128i, m: __m128i) -> __m128i {
        unsafe { _mm_blendv_epi8(a, b, m) } // sse41
    }

    #[inline(always)]
    fn i16x8_maddubs(self, a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_maddubs_epi16(a, b) } // ssse3
    }

    #[inline(always)]
    fn u32x4_blend<const IMM8: i32>(self, a: __m128i, b: __m128i) -> __m128i {
        unsafe { _mm_blend_epi32::<IMM8>(a, b) }
    }
}

unsafe impl SIMD256Ext for SSE41 {}

unsafe impl SIMD128Ext for AVX2 {
    #[inline(always)]
    fn i16x8_mul_hi(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().i16x8_mul_hi(a, b)
    }

    #[inline(always)]
    fn u16x8_mul_hi(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u16x8_mul_hi(a, b)
    }

    #[inline(always)]
    fn u8x16_blendv(self, a: Self::V128, b: Self::V128, m: Self::V128) -> Self::V128 {
        self.sse41().u8x16_blendv(a, b, m)
    }

    #[inline(always)]
    fn i16x8_maddubs(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().i16x8_maddubs(a, b)
    }

    #[inline(always)]
    fn u32x4_blend<const IMM8: i32>(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u32x4_blend::<IMM8>(a, b)
    }
}

unsafe impl SIMD256Ext for AVX2 {
    #[inline(always)]
    fn i16x16_mul_hi(self, a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_mulhi_epi16(a, b) } // avx2
    }

    #[inline(always)]
    fn u16x16_mul_hi(self, a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_mulhi_epu16(a, b) } // avx2
    }

    #[inline(always)]
    fn u8x32_blendv(self, a: __m256i, b: __m256i, m: __m256i) -> __m256i {
        unsafe { _mm256_blendv_epi8(a, b, m) } // avx2
    }

    #[inline(always)]
    fn i16x16_maddubs(self, a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_maddubs_epi16(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_blend<const IMM8: i32>(self, a: __m256i, b: __m256i) -> __m256i {
        unsafe { _mm256_blend_epi32::<IMM8>(a, b) }
    }
}
