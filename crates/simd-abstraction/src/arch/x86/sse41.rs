use super::*;

use crate::isa::{mock256, SimdLoad};

unsafe impl SIMD128 for SSE41 {
    type V128 = __m128i;

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
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
        _mm_storeu_si128(addr.cast(), a) // sse2
    }

    #[inline(always)]
    fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_or_si128(a, b) } // sse2
    }

    #[inline(always)]
    fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_and_si128(a, b) } // sse2
    }

    #[inline(always)]
    fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
        unsafe { core::mem::transmute(a) }
    }

    #[inline(always)]
    fn v128_create_zero(self) -> Self::V128 {
        unsafe { _mm_setzero_si128() } // sse2
    }

    #[inline(always)]
    fn v128_all_zero(self, a: Self::V128) -> bool {
        unsafe { _mm_testz_si128(a, a) != 0 } // sse41
    }

    #[inline(always)]
    fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_andnot_si128(b, a) } // sse2
    }

    #[inline(always)]
    fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_xor_si128(a, b) } // sse2
    }

    #[inline(always)]
    fn u8x16_splat(self, x: u8) -> Self::V128 {
        unsafe { _mm_set1_epi8(x as i8) } // sse2
    }

    #[inline(always)]
    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_shuffle_epi8(a, b) } // ssse3
    }

    #[inline(always)]
    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_subs_epu8(a, b) } // sse2
    }

    #[inline(always)]
    fn u8x16_any_zero(self, a: Self::V128) -> bool {
        unsafe {
            let cmp = _mm_cmpeq_epi8(a, _mm_setzero_si128()); // sse2
            !self.v128_all_zero(cmp)
        }
    }

    #[inline(always)]
    fn i8x16_splat(self, x: i8) -> Self::V128 {
        unsafe { _mm_set1_epi8(x) } // sse2
    }

    #[inline(always)]
    fn i8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_cmplt_epi8(a, b) } // sse2
    }

    #[inline(always)]
    fn i8x16_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_cmpeq_epi8(a, b) } // sse2
    }

    #[inline(always)]
    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { _mm_slli_epi16::<IMM8>(a) } // sse2
    }

    #[inline(always)]
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { _mm_srli_epi16::<IMM8>(a) } // sse2
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
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { _mm_slli_epi32::<IMM8>(a) } // sse2
    }

    #[inline(always)]
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { _mm_srli_epi32::<IMM8>(a) } // sse2
    }

    #[inline(always)]
    fn u32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        let a = self.u32x4_sub(a, self.u32x4_splat(u32::MAX / 2));
        let b = self.u32x4_sub(b, self.u32x4_splat(u32::MAX / 2));
        self.i32x4_lt(a, b)
    }

    #[inline(always)]
    fn i32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { _mm_cmplt_epi32(a, b) } // sse2
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

    #[inline(always)]
    fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256 {
        unsafe {
            let zero = _mm_setzero_si128(); // sse2
            (_mm_unpacklo_epi8(a, zero), _mm_unpackhi_epi8(a, zero)) // sse2
        }
    }

    #[inline(always)]
    fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
        unsafe { _mm_unpacklo_epi64(a.0, a.1) } // sse2
    }

    #[inline(always)]
    fn u8x32_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u8x32_add(self, a, b)
    }

    #[inline(always)]
    fn u16x16_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u16x16_add(self, a, b)
    }

    #[inline(always)]
    fn u32x8_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u32x8_add(self, a, b)
    }

    #[inline(always)]
    fn u64x4_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u64x4_add(self, a, b)
    }

    #[inline(always)]
    fn u8x32_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u8x32_sub(self, a, b)
    }

    #[inline(always)]
    fn u16x16_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u16x16_sub(self, a, b)
    }

    #[inline(always)]
    fn u32x8_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u32x8_sub(self, a, b)
    }

    #[inline(always)]
    fn u64x4_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u64x4_sub(self, a, b)
    }

    #[inline(always)]
    fn u8x32_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u8x32_max(self, a, b)
    }

    #[inline(always)]
    fn u16x16_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u16x16_max(self, a, b)
    }

    #[inline(always)]
    fn u32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u32x8_max(self, a, b)
    }

    #[inline(always)]
    fn i8x32_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::i8x32_max(self, a, b)
    }

    #[inline(always)]
    fn i16x16_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::i16x16_max(self, a, b)
    }

    #[inline(always)]
    fn i32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::i32x8_max(self, a, b)
    }

    #[inline(always)]
    fn u8x32_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u8x32_min(self, a, b)
    }

    #[inline(always)]
    fn u16x16_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u16x16_min(self, a, b)
    }

    #[inline(always)]
    fn u32x8_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::u32x8_min(self, a, b)
    }

    #[inline(always)]
    fn i8x32_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::i8x32_min(self, a, b)
    }

    #[inline(always)]
    fn i16x16_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::i16x16_min(self, a, b)
    }

    #[inline(always)]
    fn i32x8_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        mock256::i32x8_min(self, a, b)
    }

    #[inline(always)]
    fn u16x16_bswap(self, a: Self::V256) -> Self::V256 {
        mock256::u16x16_bswap(self, a)
    }

    #[inline(always)]
    fn u32x8_bswap(self, a: Self::V256) -> Self::V256 {
        mock256::u32x8_bswap(self, a)
    }

    #[inline(always)]
    fn u64x4_bswap(self, a: Self::V256) -> Self::V256 {
        mock256::u64x4_bswap(self, a)
    }
}
