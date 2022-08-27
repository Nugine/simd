pub use self::spec::SIMDExt;

use crate::isa::{SimdLoad, SIMD256};
use crate::scalar::Bytes32;

#[inline]
pub fn check_u8x32<S: SIMD256>(s: S, a: S::V256) -> bool {
    let a1 = s.u8x32_add(a, s.u8x32_splat(0x50));
    let a2 = s.v256_and(a1, s.u8x32_splat(0xdf));
    let a3 = s.u8x32_sub(a2, s.u8x32_splat(0x11));
    let a4 = s.i8x32_lt(a1, s.i8x32_splat(-118));
    let a5 = s.i8x32_lt(a3, s.i8x32_splat(-122));
    let a6 = s.v256_or(a4, a5);
    !s.v256_all_zero(a6)
}

fn check_u8x32_hilo<S: SIMD256>(s: S, hi: S::V256, lo: S::V256) -> bool {
    const HI_LUT: &Bytes32 = &Bytes32::double([
        0x00, 0x00, 0x00, 0x0f, 0xf0, 0x00, 0xf0, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);
    const LI_LUT: &Bytes32 = &Bytes32::double([
        0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, //
        0x0f, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);

    let hi_check = s.u8x16x2_swizzle(s.load(HI_LUT), hi);
    let lo_check = s.u8x16x2_swizzle(s.load(LI_LUT), lo);
    let check = s.v256_and(hi_check, lo_check);

    !s.u8x32_any_zero(check)
}

#[allow(clippy::result_unit_err)]
#[inline]
pub fn decode_u8x32<S: SIMDExt>(s: S, a: S::V256) -> Result<S::V128, ()> {
    let hi = s.u16x16_shr::<4>(s.v256_and(a, s.u8x32_splat(0xf0)));
    let lo = s.v256_and(a, s.u8x32_splat(0x0f));

    if !check_u8x32_hilo(s, hi, lo) {
        return Err(());
    }

    const OFFSET_LUT: &Bytes32 = &Bytes32::double([
        0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x09, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);

    const SHUFFLE: &Bytes32 = &Bytes32::double([
        0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, //
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
    ]);

    let offset = s.u8x16x2_swizzle(s.load(OFFSET_LUT), hi);

    let a1 = s.u8x32_add(lo, offset);
    let a2 = s.u16x16_shl::<4>(a1);
    let a3 = s.u16x16_shr::<12>(a2);
    let a4 = s.v256_or(a2, a3);
    let a5 = s.u8x16x2_swizzle(a4, s.load(SHUFFLE));
    let a6 = s.u64x4_unzip_low(a5);

    Ok(a6)
}

pub const ENCODE_UPPER_LUT: &Bytes32 = &Bytes32(*b"0123456789ABCDEF0123456789ABCDEF");
pub const ENCODE_LOWER_LUT: &Bytes32 = &Bytes32(*b"0123456789abcdef0123456789abcdef");

#[inline]
pub fn encode_u8x16<S: SIMDExt>(s: S, a: S::V128, lut: S::V256) -> S::V256 {
    let a0 = s.u16x16_from_u8x16(a);
    let a1 = s.u16x16_shl::<8>(a0);
    let a2 = s.u16x16_shr::<4>(a0);
    let a3 = s.v256_and(s.v256_or(a1, a2), s.u8x32_splat(0x0f));
    s.u8x16x2_swizzle(lut, a3)
}

#[inline(always)]
#[must_use]
pub const fn unhex(x: u8) -> u8 {
    const UNHEX_TABLE: &[u8; 256] = &{
        let mut buf = [0; 256];
        let mut i: usize = 0;
        while i < 256 {
            let x = i as u8;
            buf[i] = match x {
                b'0'..=b'9' => x - b'0',
                b'a'..=b'f' => x - b'a' + 10,
                b'A'..=b'F' => x - b'A' + 10,
                _ => 0xff,
            };
            i += 1
        }
        buf
    };
    UNHEX_TABLE[x as usize]
}

mod spec {
    use crate::isa::SIMD256;

    pub unsafe trait SIMDExt: SIMD256 {
        fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256;
        fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    mod x86 {
        use super::*;

        #[cfg(target_arch = "x86")]
        use core::arch::x86::*;

        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::*;

        use crate::arch::x86::*;

        unsafe impl SIMDExt for AVX2 {
            #[inline(always)]
            fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256 {
                unsafe { _mm256_cvtepu8_epi16(a) } // avx2
            }

            #[inline(always)]
            fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
                // avx2
                unsafe { _mm256_castsi256_si128(_mm256_permute4x64_epi64::<0b_0000_1000>(a)) }
            }
        }

        unsafe impl SIMDExt for SSE41 {
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
        }
    }

    #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
    mod arm {
        use super::*;

        use crate::arch::arm::*;

        #[cfg(target_arch = "arm")]
        use core::arch::arm::*;

        #[cfg(target_arch = "aarch64")]
        use core::arch::aarch64::*;

        unsafe impl SIMDExt for NEON {
            #[inline(always)]
            fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256 {
                #[cfg(target_arch = "arm")]
                unsafe {
                    let a0 = vreinterpretq_u8_u16(vmovl_u8(vget_low_u8(a)));
                    let a1 = vreinterpretq_u8_u16(vmovl_u8(vget_high_u8(a)));
                    uint8x16x2_t(a0, a1)
                }
                #[cfg(target_arch = "aarch64")]
                unsafe {
                    let f = vreinterpretq_u8_u16;
                    uint8x16x2_t(f(vmovl_u8(vget_low_u8(a))), f(vmovl_high_u8(a)))
                }
            }

            #[inline(always)]
            fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
                #[cfg(target_arch = "arm")]
                unsafe {
                    let a0 = vgetq_lane_u64::<0>(vreinterpretq_u64_u8(a.0));
                    let a1 = vgetq_lane_u64::<0>(vreinterpretq_u64_u8(a.1));
                    vreinterpretq_u8_u64(vsetq_lane_u64::<1>(a1, vdupq_n_u64(a0)))
                }
                #[cfg(target_arch = "aarch64")]
                unsafe {
                    let f = vreinterpretq_u64_u8;
                    let g = vreinterpretq_u8_u64;
                    g(vuzp1q_u64(f(a.0), f(a.1)))
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    mod wasm {
        use super::*;

        use crate::arch::wasm::*;

        use core::arch::wasm32::*;

        unsafe impl SIMDExt for SIMD128 {
            #[inline(always)]
            fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256 {
                let a0 = u16x8_extend_low_u8x16(a);
                let a1 = u16x8_extend_high_u8x16(a);
                self.v256_from_v128x2(a0, a1)
            }

            #[inline(always)]
            fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
                let a = self.v256_to_v128x2(a);
                u64x2_shuffle::<0, 2>(a.0, a.1)
            }
        }
    }
}
