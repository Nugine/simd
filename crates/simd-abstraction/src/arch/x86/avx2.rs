use super::*;

impl AVX2 {
    #[inline(always)]
    fn sse41(self) -> SSE41 {
        unsafe { SSE41::new() }
    }
}

unsafe impl SIMD128 for AVX2 {
    type V128 = <SSE41 as SIMD128>::V128;

    #[inline(always)]
    unsafe fn v128_load(self, addr: *const u8) -> Self::V128 {
        self.sse41().v128_load(addr)
    }

    #[inline(always)]
    unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128 {
        self.sse41().v128_load_unaligned(addr)
    }

    #[inline(always)]
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
        self.sse41().v128_store_unaligned(addr, a)
    }

    #[inline(always)]
    fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().v128_or(a, b)
    }

    #[inline(always)]
    fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().v128_and(a, b)
    }

    #[inline(always)]
    fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
        self.sse41().v128_to_bytes(a)
    }

    #[inline(always)]
    fn v128_create_zero(self) -> Self::V128 {
        self.sse41().v128_create_zero()
    }

    #[inline(always)]
    fn v128_all_zero(self, a: Self::V128) -> bool {
        self.sse41().v128_all_zero(a)
    }

    #[inline(always)]
    fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().v128_andnot(a, b)
    }

    #[inline(always)]
    fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().v128_xor(a, b)
    }

    #[inline(always)]
    fn u8x16_splat(self, x: u8) -> Self::V128 {
        self.sse41().u8x16_splat(x)
    }

    #[inline(always)]
    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_swizzle(a, b)
    }

    #[inline(always)]
    fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_add(a, b)
    }

    #[inline(always)]
    fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_sub(a, b)
    }

    #[inline(always)]
    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_sub_sat(a, b)
    }

    #[inline(always)]
    fn u8x16_any_zero(self, a: Self::V128) -> bool {
        self.sse41().u8x16_any_zero(a)
    }

    #[inline(always)]
    fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_min(a, b)
    }

    #[inline(always)]
    fn i8x16_splat(self, x: i8) -> Self::V128 {
        self.sse41().i8x16_splat(x)
    }

    #[inline(always)]
    fn i8x16_cmp_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().i8x16_cmp_lt(a, b)
    }

    #[inline(always)]
    fn i8x16_cmp_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().i8x16_cmp_eq(a, b)
    }

    #[inline(always)]
    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        self.sse41().u16x8_shl::<IMM8>(a)
    }

    #[inline(always)]
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        self.sse41().u16x8_shr::<IMM8>(a)
    }

    #[inline(always)]
    fn u16x8_splat(self, x: u16) -> Self::V128 {
        self.sse41().u16x8_splat(x)
    }

    #[inline(always)]
    fn u32x4_splat(self, x: u32) -> Self::V128 {
        self.sse41().u32x4_splat(x)
    }

    #[inline(always)]
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        self.sse41().u32x4_shl::<IMM8>(a)
    }

    #[inline(always)]
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        self.sse41().u32x4_shr::<IMM8>(a)
    }

    #[inline(always)]
    fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u32x4_add(a, b)
    }

    #[inline(always)]
    fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u32x4_sub(a, b)
    }

    #[inline(always)]
    fn u32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u32x4_max(a, b)
    }

    #[inline(always)]
    fn u32x4_cmp_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u32x4_cmp_lt(a, b)
    }

    #[inline(always)]
    fn i32x4_cmp_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().i32x4_cmp_lt(a, b)
    }
}

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
    fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256 {
        unsafe { _mm256_cvtepu8_epi16(a) } // avx2
    }

    #[inline(always)]
    fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
        // avx2
        unsafe { _mm256_castsi256_si128(_mm256_permute4x64_epi64::<0b_0000_1000>(a)) }
    }

    #[inline(always)]
    unsafe fn v256_load(self, addr: *const u8) -> Self::V256 {
        _mm256_load_si256(addr.cast()) // avx
    }

    #[inline(always)]
    unsafe fn v256_load_unaligned(self, addr: *const u8) -> Self::V256 {
        _mm256_loadu_si256(addr.cast()) // avx
    }

    #[inline(always)]
    unsafe fn v256_store_unaligned(self, addr: *mut u8, a: Self::V256) {
        _mm256_storeu_si256(addr.cast(), a) // avx
    }

    #[inline(always)]
    fn v256_or(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_or_si256(a, b) } // avx2
    }

    #[inline(always)]
    fn v256_and(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_and_si256(a, b) } // avx2
    }

    #[inline(always)]
    fn v256_andnot(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_andnot_si256(b, a) } // avx2
    }

    #[inline(always)]
    fn v256_xor(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_xor_si256(a, b) } // avx
    }

    #[inline(always)]
    fn v256_create_zero(self) -> Self::V256 {
        unsafe { _mm256_setzero_si256() } // avx
    }

    #[inline(always)]
    fn v256_all_zero(self, a: Self::V256) -> bool {
        unsafe { _mm256_testz_si256(a, a) != 0 } // avx
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
    fn u8x32_splat(self, x: u8) -> Self::V256 {
        unsafe { _mm256_set1_epi8(x as i8) } // avx
    }

    #[inline(always)]
    fn u8x32_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_add_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn u8x32_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_sub_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn u8x32_any_zero(self, a: Self::V256) -> bool {
        unsafe {
            let cmp = _mm256_cmpeq_epi8(a, _mm256_setzero_si256()); // avx2
            _mm256_movemask_epi8(cmp) as u32 != 0 // avx2
        }
    }

    #[inline(always)]
    fn u8x16x2_swizzle(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_shuffle_epi8(a, b) } // avx2
    }

    #[inline(always)]
    fn i8x32_splat(self, x: i8) -> Self::V256 {
        unsafe { _mm256_set1_epi8(x) } // avx
    }

    #[inline(always)]
    fn i8x32_cmp_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpgt_epi8(b, a) } // avx2
    }

    #[inline(always)]
    fn i8x32_cmp_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpeq_epi8(b, a) } // avx2
    }

    #[inline(always)]
    fn u16x16_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_slli_epi16::<IMM8>(a) } // avx2
    }

    #[inline(always)]
    fn u16x16_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_srli_epi16::<IMM8>(a) } // avx2
    }

    #[inline(always)]
    fn u16x16_splat(self, x: u16) -> Self::V256 {
        unsafe { _mm256_set1_epi16(x as i16) } // avx
    }

    #[inline(always)]
    fn u32x8_splat(self, x: u32) -> Self::V256 {
        unsafe { _mm256_set1_epi32(x as i32) } // avx2
    }

    #[inline(always)]
    fn u8x32_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_subs_epu8(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_slli_epi32::<IMM8>(a) } // avx2
    }

    #[inline(always)]
    fn u32x8_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_srli_epi32::<IMM8>(a) } // avx2
    }

    #[inline(always)]
    fn u32x8_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_add_epi32(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_sub_epi32(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_max_epu32(a, b) } // avx2
    }

    #[inline(always)]
    fn u32x8_cmp_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        let a = self.u32x8_sub(a, self.u32x8_splat(u32::MAX / 2));
        let b = self.u32x8_sub(b, self.u32x8_splat(u32::MAX / 2));
        self.i32x8_cmp_lt(a, b)
    }

    #[inline(always)]
    fn i32x8_cmp_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpgt_epi32(b, a) } // avx2
    }
}
