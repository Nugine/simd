#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    use crate::generic::SIMDExt;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    use simd_abstraction::arch::x86::*;

    unsafe impl SIMDExt for AVX2 {
        #[inline(always)]
        fn i16x16_set_lane7(self, a: Self::V256, x: i16) -> Self::V256 {
            unsafe { _mm256_insert_epi16::<7>(a, x) }
        }

        #[inline(always)]
        fn i32x8_set_lane7(self, a: Self::V256, x: i32) -> Self::V256 {
            unsafe { _mm256_insert_epi32::<7>(a, x) }
        }

        #[inline(always)]
        fn i32x4_get_lane3(self, a: Self::V128) -> i32 {
            unsafe { _mm_extract_epi32::<3>(a) }
        }

        #[inline(always)]
        fn i16x8_get_lane7(self, a: Self::V128) -> i16 {
            unsafe { _mm_extract_epi16::<7>(a) as i16 }
        }
    }

    unsafe impl SIMDExt for SSE41 {
        #[inline(always)]
        fn i16x16_set_lane7(self, a: Self::V256, x: i16) -> Self::V256 {
            unsafe { (_mm_insert_epi16::<7>(a.0, x as i32), a.1) }
        }

        #[inline(always)]
        fn i32x8_set_lane7(self, a: Self::V256, x: i32) -> Self::V256 {
            unsafe { (a.0, _mm_insert_epi32::<3>(a.1, x)) }
        }

        #[inline(always)]
        fn i32x4_get_lane3(self, a: Self::V128) -> i32 {
            unsafe { _mm_extract_epi32::<3>(a) }
        }

        #[inline(always)]
        fn i16x8_get_lane7(self, a: Self::V128) -> i16 {
            unsafe { _mm_extract_epi16::<7>(a) as i16 }
        }
    }
}

#[cfg(all(
    feature = "unstable",
    any(target_arch = "arm", target_arch = "aarch64")
))]
mod arm {
    use crate::generic::SIMDExt;

    #[cfg(target_arch = "arm")]
    use core::arch::arm::*;

    #[cfg(target_arch = "aarch64")]
    use core::arch::aarch64::*;

    #[cfg(target_arch = "arm")]
    use simd_abstraction::arch::arm::*;

    #[cfg(target_arch = "aarch64")]
    use simd_abstraction::arch::aarch64::*;

    unsafe impl SIMDExt for NEON {
        #[inline(always)]
        fn i16x16_set_lane7(self, a: Self::V256, x: i16) -> Self::V256 {
            let f = vreinterpretq_s16_u8;
            let g = vreinterpretq_u8_s16;
            unsafe { uint8x16x2_t(g(vsetq_lane_s16::<7>(x, f(a.0))), a.1) }
        }

        #[inline(always)]
        fn i32x8_set_lane7(self, a: Self::V256, x: i32) -> Self::V256 {
            let f = vreinterpretq_s32_u8;
            let g = vreinterpretq_u8_s32;
            unsafe { uint8x16x2_t(a.0, g(vsetq_lane_s32::<3>(x, f(a.1)))) }
        }

        #[inline(always)]
        fn i32x4_get_lane3(self, a: Self::V128) -> i32 {
            unsafe { vgetq_lane_s32::<3>(vreinterpretq_s32_u8(a)) }
        }

        #[inline(always)]
        fn i16x8_get_lane7(self, a: Self::V128) -> i16 {
            unsafe { vgetq_lane_s16::<7>(vreinterpretq_s16_u8(a)) }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::generic::SIMDExt;

    #[cfg(target_arch = "wasm32")]
    use core::arch::wasm32::*;

    use simd_abstraction::arch::wasm::*;

    unsafe impl SIMDExt for SIMD128 {
        #[inline(always)]
        fn i16x16_set_lane7(self, a: Self::V256, x: i16) -> Self::V256 {
            (i16x8_replace_lane::<7>(a.0, x), a.1)
        }

        #[inline(always)]
        fn i32x8_set_lane7(self, a: Self::V256, x: i32) -> Self::V256 {
            (a.0, i32x4_replace_lane::<3>(a.1, x))
        }

        #[inline(always)]
        fn i32x4_get_lane3(self, a: Self::V128) -> i32 {
            i32x4_extract_lane::<3>(a)
        }

        #[inline(always)]
        fn i16x8_get_lane7(self, a: Self::V128) -> i16 {
            i16x8_extract_lane::<7>(a)
        }
    }
}
