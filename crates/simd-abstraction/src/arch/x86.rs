use crate::crc32::{CRC32, POLYNOMIAL_CRC32C};
use crate::traits::{InstructionSet, SIMD128, SIMD256};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

define_isa!(SSE41, "sse4.1", is_x86_feature_detected);
define_isa!(SSE42, "sse4.2", is_x86_feature_detected);
define_isa!(AVX2, "avx2", is_x86_feature_detected);

macro_rules! impl_simd128 {
    ($ty:ty) => {
        unsafe impl SIMD128 for $ty {
            type V128 = __m128i;

            #[inline(always)]
            unsafe fn v128_load(self, addr: *const u8) -> Self::V128 {
                debug_assert_ptr_align!(addr, 16);
                _mm_load_si128(addr.cast())
            }

            #[inline(always)]
            unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128 {
                _mm_loadu_si128(addr.cast())
            }

            #[inline(always)]
            unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
                _mm_storeu_si128(addr.cast(), a)
            }

            #[inline(always)]
            fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_or_si128(a, b) }
            }

            #[inline(always)]
            fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_and_si128(a, b) }
            }

            #[inline(always)]
            fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_andnot_si128(b, a) }
            }

            #[inline(always)]
            fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_xor_si128(a, b) }
            }

            #[inline(always)]
            fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
                unsafe { core::mem::transmute(a) }
            }

            #[inline(always)]
            fn v128_create_zero(self) -> Self::V128 {
                unsafe { _mm_setzero_si128() }
            }

            #[inline(always)]
            fn v128_all_zero(self, a: Self::V128) -> bool {
                unsafe { _mm_testz_si128(a, a) != 0 }
            }

            #[inline(always)]
            fn u8x16_splat(self, x: u8) -> Self::V128 {
                unsafe { _mm_set1_epi8(x as i8) }
            }

            #[inline(always)]
            fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_shuffle_epi8(a, b) }
            }

            #[inline(always)]
            fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_add_epi8(a, b) }
            }

            #[inline(always)]
            fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_sub_epi8(a, b) }
            }

            #[inline(always)]
            fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_subs_epu8(a, b) }
            }

            #[inline(always)]
            fn u8x16_any_zero(self, a: Self::V128) -> bool {
                unsafe {
                    let cmp = _mm_cmpeq_epi8(a, _mm_setzero_si128());
                    !self.v128_all_zero(cmp)
                }
            }

            #[inline(always)]
            fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_min_epu8(a, b) }
            }

            #[inline(always)]
            fn i8x16_splat(self, x: i8) -> Self::V128 {
                unsafe { _mm_set1_epi8(x) }
            }

            #[inline(always)]
            fn i8x16_cmp_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_cmplt_epi8(a, b) }
            }

            #[inline(always)]
            fn i8x16_cmp_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                unsafe { _mm_cmpeq_epi8(a, b) }
            }

            #[inline(always)]
            fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                unsafe { _mm_slli_epi16::<IMM8>(a) }
            }

            #[inline(always)]
            fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                unsafe { _mm_srli_epi16::<IMM8>(a) }
            }

            #[inline(always)]
            fn u16x8_splat(self, x: u16) -> Self::V128 {
                unsafe { _mm_set1_epi16(x as i16) }
            }

            #[inline(always)]
            fn u32x4_splat(self, x: u32) -> Self::V128 {
                unsafe { _mm_set1_epi32(x as i32) }
            }

            #[inline(always)]
            fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                unsafe { _mm_slli_epi32::<IMM8>(a) }
            }

            #[inline(always)]
            fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                unsafe { _mm_srli_epi32::<IMM8>(a) }
            }
        }
    };
}

impl_simd128!(SSE41);
impl_simd128!(SSE42);
impl_simd128!(AVX2);

macro_rules! mock_simd256 {
    ($ty:ty) => {
        unsafe impl SIMD256 for $ty {
            type V256 = (__m128i, __m128i);

            #[inline(always)]
            fn v256_to_bytes(self, a: Self::V256) -> [u8; 32] {
                unsafe { core::mem::transmute([a.0, a.1]) }
            }

            #[inline(always)]
            fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256 {
                unsafe {
                    let zero = _mm_setzero_si128();
                    (_mm_unpacklo_epi8(a, zero), _mm_unpackhi_epi8(a, zero))
                }
            }

            #[inline(always)]
            fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
                unsafe { _mm_unpacklo_epi64(a.0, a.1) }
            }

            #[inline(always)]
            fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256 {
                (a, b)
            }

            #[inline(always)]
            fn v256_to_v128x2(self, a: Self::V256) -> (Self::V128, Self::V128) {
                (a.0, a.1)
            }
        }
    };
}

mock_simd256!(SSE41);
mock_simd256!(SSE42);

unsafe impl SIMD256 for AVX2 {
    type V256 = __m256i;

    #[inline(always)]
    fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256 {
        unsafe { _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(a), b) }
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
        unsafe { _mm256_cvtepu8_epi16(a) }
    }

    #[inline(always)]
    fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
        unsafe { _mm256_castsi256_si128(_mm256_permute4x64_epi64::<0b_0000_1000>(a)) }
    }

    #[inline(always)]
    unsafe fn v256_load(self, addr: *const u8) -> Self::V256 {
        _mm256_load_si256(addr.cast())
    }

    #[inline(always)]
    unsafe fn v256_load_unaligned(self, addr: *const u8) -> Self::V256 {
        _mm256_loadu_si256(addr.cast())
    }

    #[inline(always)]
    unsafe fn v256_store_unaligned(self, addr: *mut u8, a: Self::V256) {
        _mm256_storeu_si256(addr.cast(), a)
    }

    #[inline(always)]
    fn v256_or(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_or_si256(a, b) }
    }

    #[inline(always)]
    fn v256_and(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_and_si256(a, b) }
    }

    #[inline(always)]
    fn v256_andnot(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_andnot_si256(b, a) }
    }

    #[inline(always)]
    fn v256_xor(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_xor_si256(a, b) }
    }

    #[inline(always)]
    fn v256_create_zero(self) -> Self::V256 {
        unsafe { _mm256_setzero_si256() }
    }

    #[inline(always)]
    fn v256_all_zero(self, a: Self::V256) -> bool {
        unsafe { _mm256_testz_si256(a, a) != 0 }
    }

    #[inline(always)]
    fn v256_get_low(self, a: Self::V256) -> Self::V128 {
        unsafe { _mm256_castsi256_si128(a) }
    }

    #[inline(always)]
    fn v256_get_high(self, a: Self::V256) -> Self::V128 {
        unsafe { _mm256_extracti128_si256::<1>(a) }
    }

    #[inline(always)]
    fn u8x32_splat(self, x: u8) -> Self::V256 {
        unsafe { _mm256_set1_epi8(x as i8) }
    }

    #[inline(always)]
    fn u8x32_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_add_epi8(a, b) }
    }

    #[inline(always)]
    fn u8x32_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_sub_epi8(a, b) }
    }

    #[inline(always)]
    fn u8x32_any_zero(self, a: Self::V256) -> bool {
        unsafe {
            let cmp = _mm256_cmpeq_epi8(a, _mm256_setzero_si256());
            _mm256_movemask_epi8(cmp) as u32 != 0
        }
    }

    #[inline(always)]
    fn u8x16x2_swizzle(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_shuffle_epi8(a, b) }
    }

    #[inline(always)]
    fn i8x32_splat(self, x: i8) -> Self::V256 {
        unsafe { _mm256_set1_epi8(x) }
    }

    #[inline(always)]
    fn i8x32_cmp_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpgt_epi8(b, a) }
    }

    #[inline(always)]
    fn i8x32_cmp_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_cmpeq_epi8(b, a) }
    }

    #[inline(always)]
    fn u16x16_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_slli_epi16::<IMM8>(a) }
    }

    #[inline(always)]
    fn u16x16_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_srli_epi16::<IMM8>(a) }
    }

    #[inline(always)]
    fn u16x16_splat(self, x: u16) -> Self::V256 {
        unsafe { _mm256_set1_epi16(x as i16) }
    }

    #[inline(always)]
    fn u32x8_splat(self, x: u32) -> Self::V256 {
        unsafe { _mm256_set1_epi32(x as i32) }
    }

    #[inline(always)]
    fn u8x32_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        unsafe { _mm256_subs_epu8(a, b) }
    }

    #[inline(always)]
    fn u32x8_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_slli_epi32::<IMM8>(a) }
    }

    #[inline(always)]
    fn u32x8_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        unsafe { _mm256_srli_epi32::<IMM8>(a) }
    }
}

unsafe impl CRC32<POLYNOMIAL_CRC32C> for SSE42 {
    #[inline(always)]
    fn crc32_u8(self, crc: u32, value: u8) -> u32 {
        unsafe { _mm_crc32_u8(crc, value) }
    }

    #[inline(always)]
    fn crc32_u16(self, crc: u32, value: u16) -> u32 {
        unsafe { _mm_crc32_u16(crc, value) }
    }

    #[inline(always)]
    fn crc32_u32(self, crc: u32, value: u32) -> u32 {
        unsafe { _mm_crc32_u32(crc, value) }
    }

    #[inline(always)]
    fn crc32_u64(self, crc: u32, value: u64) -> u32 {
        unsafe { _mm_crc32_u64(crc as u64, value) as u32 }
    }
}
