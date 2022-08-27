pub use self::spec::SIMDExt;

use crate::isa::SimdLoad;
use crate::scalar::Bytes32;
use crate::tools::{read, write};

#[inline(always)]
pub unsafe fn encode_bits<const N: usize>(dst: *mut u8, charset: *const u8, x: u64) {
    debug_assert!(matches!(N, 2 | 4 | 5 | 7 | 8));

    {
        let shift = (N - 1) * 5;
        write(dst, 0, read(charset, (x >> shift) as usize));
    }
    for i in 1..N {
        let shift = (N - 1 - i) * 5;
        write(dst, i, read(charset, ((x >> shift) & 0x1f) as usize));
    }
}

#[inline(always)]
pub unsafe fn decode_bits<const N: usize>(src: *const u8, table: *const u8) -> (u64, u8) {
    debug_assert!(matches!(N, 2 | 4 | 5 | 7 | 8));
    let mut ans: u64 = 0;
    let mut flag = 0;
    for i in 0..N {
        let bits = read(table, read(src, i) as usize);
        flag |= bits;
        ans = (ans << 5) | u64::from(bits);
    }
    (ans, flag)
}

#[inline(always)]
pub unsafe fn read_be_bytes<const N: usize>(src: *const u8) -> u64 {
    debug_assert!(matches!(N, 1 | 2 | 3 | 4 | 5));

    #[cfg(not(target_arch = "wasm32"))]
    {
        if N == 3 {
            let x1: u64 = read(src, 0).into();
            let x2: u64 = src.add(1).cast::<u16>().read_unaligned().to_be().into();
            return (x1 << 16) | x2;
        }
        if N == 5 {
            let x1: u64 = read(src, 0).into();
            let x2: u64 = src.add(1).cast::<u64>().read_unaligned().to_be();
            return (x1 << 32) | x2;
        }
    }

    let mut ans = 0;
    for i in 0..N {
        let shift = (N - 1 - i) * 8;
        ans |= u64::from(read(src, i)) << shift;
    }
    ans
}

#[inline(always)]
pub unsafe fn write_be_bytes<const N: usize>(dst: *mut u8, x: u64) {
    debug_assert!(matches!(N, 1 | 2 | 3 | 4 | 5));

    #[cfg(not(target_arch = "wasm32"))]
    {
        if N == 3 {
            let x1 = (x >> 16) as u8;
            let x2 = (x as u16).to_be();
            dst.write(x1);
            dst.add(1).cast::<u16>().write_unaligned(x2);
            return;
        }
        if N == 5 {
            let x1 = (x >> 32) as u8;
            let x2 = (x as u32).to_be();
            dst.write(x1);
            dst.add(1).cast::<u32>().write_unaligned(x2);
            return;
        }
    }

    for i in 0..N {
        let shift = (N - 1 - i) * 8;
        write(dst, i, (x >> shift) as u8);
    }
}

#[allow(clippy::result_unit_err)]
#[inline(always)]
pub fn rfc4648_decode_bits_simd<S: SIMDExt>(
    s: S,
    x: S::V256,
    ch0: u8,
    len0: u8,
    ch1: u8,
    len1: u8,
) -> Result<S::V256, ()> {
    let x1 = s.u8x32_sub(x, s.u8x32_splat(ch0));
    let m1 = s.u8x32_lt(x1, s.u8x32_splat(len1));

    let x2 = s.u8x32_sub(x, s.u8x32_splat(ch1));
    let m2 = s.u8x32_lt(x2, s.u8x32_splat(len1));

    if s.u8x32_any_zero(s.v256_or(m1, m2)) {
        return Err(());
    }

    let r0 = s.v256_and(x1, m1);
    let r1 = s.v256_and(s.u8x32_add(x2, s.u8x32_splat(len0)), m2);

    Ok(s.base32_merge_bits(s.v256_or(r0, r1)))
}

#[allow(clippy::result_unit_err)]
#[inline(always)]
pub fn crockford_decode_bits_simd<S: SIMDExt>(s: S, x: S::V256) -> Result<S::V256, ()> {
    const M3: &Bytes32 = &Bytes32::double([
        0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);

    const SHIFT: &Bytes32 = &Bytes32::double([
        0x00, 0xfe, 0xfe, 0xfe, 0xfe, //
        0x00, 0xfd, 0xfd, 0xfd, 0xfd, 0xfd, 0xfd, 0xfd, 0xfd, 0xfd, 0xfd, //
    ]);

    let (ch0, len0, ch1, len1) = (b'0', 10, b'A', 26);

    let x1 = s.u8x32_sub(x, s.u8x32_splat(ch0));
    let m1 = s.u8x32_lt(x1, s.u8x32_splat(len1));

    let x2 = s.u8x32_sub(x, s.u8x32_splat(ch1));
    let m2 = s.u8x32_lt(x2, s.u8x32_splat(len1));

    let x3 = s.u8x32_sub(x, s.u8x32_splat(b'I'));
    let m3 = s.u8x16x2_swizzle(s.load(M3), x3);

    if s.u8x32_any_zero(s.v256_or(m1, s.v256_andnot(m2, m3))) {
        return Err(());
    }

    let x4 = s.u8x32_add(x3, s.u8x32_splat(1));
    let shift = s.u8x16x2_swizzle(s.load(SHIFT), x4);

    let r0 = s.v256_and(x1, m1);
    let x5 = s.u8x32_add(x2, s.u8x32_splat(len0));
    let r1 = s.u8x32_add(s.v256_and(x5, m2), shift);

    Ok(s.base32_merge_bits(s.v256_or(r0, r1)))
}

mod spec {
    use crate::isa::{SimdLoad, SIMD256};
    use crate::scalar::Bytes32;

    const fn u16x4_to_u64(x: [u16; 4]) -> u64 {
        unsafe { core::mem::transmute(x) }
    }

    const SPLIT_SHUFFLE: &Bytes32 = &Bytes32([
        0x07, 0x06, 0x08, 0x07, 0x09, 0x08, 0x0A, 0x09, //
        0x0C, 0x0B, 0x0D, 0x0C, 0x0E, 0x0D, 0x0F, 0x0E, //
        0x01, 0x00, 0x02, 0x01, 0x03, 0x02, 0x04, 0x03, //
        0x06, 0x05, 0x07, 0x06, 0x08, 0x07, 0x09, 0x08, //
    ]);

    pub unsafe trait SIMDExt: SIMD256 {
        #[inline(always)]
        fn base32_split_bits(self, x: Self::V256) -> Self::V256 {
            const SPLIT_M1: u64 = u16x4_to_u64([1 << 1, 1 << 3, 1 << 5, 1 << 7]);
            const SPLIT_M2: u64 = u16x4_to_u64([1 << 2, 1 << 4, 1 << 6, 1 << 8]);
            const SPLIT_M3: u16 = u16::from_le_bytes([0x00, 0x1f]);

            let x1 = self.u8x16x2_swizzle(x, self.load(SPLIT_SHUFFLE));
            let x2 = self.u16x16_shr::<4>(x1);
            let x3 = self.i16x16_mul_lo(x2, self.u64x4_splat(SPLIT_M1));
            let x4 = self.i16x16_mul_lo(x1, self.u64x4_splat(SPLIT_M2));
            let m3 = self.u16x16_splat(SPLIT_M3);
            let x5 = self.v256_and(x3, m3);
            let x6 = self.v256_and(x4, m3);
            let x7 = self.u16x16_shr::<8>(x5);
            self.v256_or(x6, x7)
        }

        fn base32_encode_symbols(self, x: Self::V256, lut0: Self::V256, lut1: Self::V256) -> Self::V256;

        #[inline(always)]
        fn base32_merge_bits(self, x: Self::V256) -> Self::V256 {
            const MERGE_M1: u16 = u16::from_le_bytes([0x1f, 0x00]);
            const MERGE_M2: u64 = u16x4_to_u64([1 << 3, 1 << 1, 1 << 7, 1 << 5]);
            const MERGE_M3: u64 = u16x4_to_u64([1 << 6, 1 << 4, 1 << 2, 1 << 0]);

            const MERGE_S1: &Bytes32 = &Bytes32::double([
                0x00, 0x02, 0x05, 0x07, 0x06, 0x80, 0x80, 0x04, //
                0x08, 0x0A, 0x0D, 0x0F, 0x0E, 0x80, 0x80, 0x0C, //
            ]);
            const MERGE_S2: &Bytes32 = &Bytes32::double([
                0x01, 0x00, 0x02, 0x04, 0x06, 0x03, 0x80, 0x80, //
                0x09, 0x08, 0x0A, 0x0C, 0x0E, 0x0B, 0x80, 0x80, //
            ]);
            const MERGE_S3: &Bytes32 = &Bytes32::double([
                0x00, 0x01, 0x02, 0x03, 0x04, //
                0x08, 0x09, 0x0A, 0x0B, 0x0C, //
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
            ]);
            const MERGE_S4: &Bytes32 = &Bytes32::double([
                0x80, 0x05, 0x80, 0x07, 0x80, //
                0x80, 0x0D, 0x80, 0x0F, 0x80, //
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
            ]);

            let x1 = self.v256_and(x, self.u16x16_splat(MERGE_M1));
            let x2 = self.i16x16_mul_lo(x1, self.u64x4_splat(MERGE_M2));
            let x3 = self.u16x16_shr::<8>(x);
            let x4 = self.i16x16_mul_lo(x3, self.u64x4_splat(MERGE_M3));
            let x5 = self.u8x16x2_swizzle(x2, self.load(MERGE_S1));
            let x6 = self.u8x16x2_swizzle(x4, self.load(MERGE_S2));
            let x7 = self.v256_or(x5, x6);
            let x8 = self.u8x16x2_swizzle(x7, self.load(MERGE_S3));
            let x9 = self.u8x16x2_swizzle(x7, self.load(MERGE_S4));
            self.v256_or(x8, x9)
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    mod x86 {
        use super::*;

        use crate::arch::x86::*;

        #[inline(always)]
        fn base32_split_bits<S: SIMD256Ext>(s: S, x: S::V256) -> S::V256 {
            const SPLIT_M1: u64 = u16x4_to_u64([1 << 5, 1 << 7, 1 << 9, 1 << 11]);
            const SPLIT_M2: u64 = u16x4_to_u64([1 << 2, 1 << 4, 1 << 6, 1 << 8]);
            const SPLIT_M3: u16 = u16::from_le_bytes([0x00, 0x1f]);

            let x1 = s.u8x16x2_swizzle(x, s.load(SPLIT_SHUFFLE));
            let x2 = s.u16x16_mul_hi(x1, s.u64x4_splat(SPLIT_M1));
            let x3 = s.i16x16_mul_lo(x1, s.u64x4_splat(SPLIT_M2));
            let x4 = s.v256_and(x3, s.u16x16_splat(SPLIT_M3));
            s.v256_or(x2, x4)
        }

        #[inline(always)]
        fn base32_encode_symbols<S: SIMD256Ext>(s: S, x: S::V256, lut0: S::V256, lut1: S::V256) -> S::V256 {
            let x1 = s.u8x16x2_swizzle(lut0, x);
            let x2 = s.u8x16x2_swizzle(lut1, x);
            let x3 = s.v256_and(x, s.u8x32_splat(0x10));
            s.u8x32_blendv(x1, x2, x3)
        }

        #[inline(always)]
        fn base32_merge_bits<S: SIMD256Ext>(s: S, x: S::V256) -> S::V256 {
            const MERGE_M1: u32 = u32::from_le_bytes([1 << 7, 1 << 2, 1 << 5, 1 << 0]);
            const MERGE_S1: &Bytes32 = &Bytes32::double([
                0x01, 0x00, 0x02, 0x04, 0x06, //
                0x09, 0x08, 0x0A, 0x0C, 0x0E, //
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
            ]);
            const MERGE_S2: &Bytes32 = &Bytes32::double([
                0x80, 0x03, 0x05, 0x07, 0x80, //
                0x80, 0x0B, 0x0D, 0x0F, 0x80, //
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
            ]);

            let x1 = s.i16x16_maddubs(x, s.u32x8_splat(MERGE_M1));
            let x2 = s.u32x8_shl::<4>(x1);
            let x3 = s.u32x8_blend::<0x55>(x1, x2);
            let x4 = s.u8x16x2_swizzle(x3, s.load(MERGE_S1));
            let x5 = s.u8x16x2_swizzle(x3, s.load(MERGE_S2));
            s.v256_or(x4, x5)
        }

        unsafe impl SIMDExt for AVX2 {
            #[inline(always)]
            fn base32_split_bits(self, x: Self::V256) -> Self::V256 {
                base32_split_bits(self, x)
            }

            #[inline(always)]
            fn base32_encode_symbols(self, x: Self::V256, lut0: Self::V256, lut1: Self::V256) -> Self::V256 {
                base32_encode_symbols(self, x, lut0, lut1)
            }

            #[inline(always)]
            fn base32_merge_bits(self, x: Self::V256) -> Self::V256 {
                base32_merge_bits(self, x)
            }
        }

        unsafe impl SIMDExt for SSE41 {
            #[inline(always)]
            fn base32_split_bits(self, x: Self::V256) -> Self::V256 {
                base32_split_bits(self, x)
            }

            #[inline(always)]
            fn base32_encode_symbols(self, x: Self::V256, lut0: Self::V256, lut1: Self::V256) -> Self::V256 {
                base32_encode_symbols(self, x, lut0, lut1)
            }

            #[inline(always)]
            fn base32_merge_bits(self, x: Self::V256) -> Self::V256 {
                base32_merge_bits(self, x)
            }
        }
    }

    #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
    mod arm {
        use super::*;

        use crate::arch::arm::*;

        unsafe impl SIMDExt for NEON {
            #[inline(always)]
            fn base32_encode_symbols(self, x: Self::V256, lut0: Self::V256, lut1: Self::V256) -> Self::V256 {
                #[cfg(target_arch = "aarch64")]
                {
                    let lut = self.v256_from_v128x2(lut0.0, lut1.0);
                    self.u8x32_swizzle(lut, x)
                    // 32B table lookup
                }
                #[cfg(target_arch = "arm")]
                {
                    let x1 = self.u8x16x2_swizzle(lut0, x);
                    let x2 = self.u8x16x2_swizzle(lut1, x);
                    let x3 = self.u8x32_lt(self.u8x32_splat(0x0f), x);
                    self.v256_bsl(x3, x2, x1)
                    // for each bit: if x3 == 1 { x2 } else { x1 }
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    mod wasm {
        use super::*;

        use crate::arch::wasm::*;

        unsafe impl SIMDExt for SIMD128 {
            #[inline(always)]
            fn base32_encode_symbols(self, x: Self::V256, lut0: Self::V256, lut1: Self::V256) -> Self::V256 {
                let x1 = self.u8x16x2_swizzle(lut0, x);
                let x2 = self.u8x16x2_swizzle(lut1, x);
                let x3 = self.u8x32_lt(self.u8x32_splat(0x0f), x);
                self.v256_bsl(x3, x2, x1)
                // for each bit: if x3 == 1 { x2 } else { x1 }
            }
        }
    }
}
