use crate::simd256::simd256_vop2;
use crate::tools::{read, write};
use crate::{AVX2, NEON, SIMD256, SSE41, V256, WASM128};

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
#[must_use]
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
#[must_use]
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

#[inline(always)]
const fn u16x4_to_u64(x: [u16; 4]) -> u64 {
    unsafe { core::mem::transmute(x) }
}

#[inline(always)]
pub fn split_bits_simd<S: SIMD256>(s: S, x: V256) -> V256 {
    const SPLIT_SHUFFLE: V256 = V256::from_bytes([
        0x07, 0x06, 0x08, 0x07, 0x09, 0x08, 0x0A, 0x09, //
        0x0C, 0x0B, 0x0D, 0x0C, 0x0E, 0x0D, 0x0F, 0x0E, //
        0x01, 0x00, 0x02, 0x01, 0x03, 0x02, 0x04, 0x03, //
        0x06, 0x05, 0x07, 0x06, 0x08, 0x07, 0x09, 0x08, //
    ]);

    if is_subtype!(S, SSE41) {
        const SPLIT_M1: u64 = u16x4_to_u64([1 << 5, 1 << 7, 1 << 9, 1 << 11]);
        const SPLIT_M2: u64 = u16x4_to_u64([1 << 2, 1 << 4, 1 << 6, 1 << 8]);
        const SPLIT_M3: u16 = u16::from_le_bytes([0x00, 0x1f]);

        let x1 = s.u8x16x2_swizzle(x, SPLIT_SHUFFLE);
        let x2 = s.u16x16_mul_hi(x1, s.u64x4_splat(SPLIT_M1));
        let x3 = s.i16x16_mul_lo(x1, s.u64x4_splat(SPLIT_M2));
        let x4 = s.v256_and(x3, s.u16x16_splat(SPLIT_M3));
        return s.v256_or(x2, x4);
    }

    if is_subtype!(S, NEON | WASM128) {
        const SPLIT_M1: u64 = u16x4_to_u64([1 << 1, 1 << 3, 1 << 5, 1 << 7]);
        const SPLIT_M2: u64 = u16x4_to_u64([1 << 2, 1 << 4, 1 << 6, 1 << 8]);
        const SPLIT_M3: u16 = u16::from_le_bytes([0x00, 0x1f]);

        let x1 = s.u8x16x2_swizzle(x, SPLIT_SHUFFLE);
        let x2 = s.u16x16_shr::<4>(x1);
        let x3 = s.i16x16_mul_lo(x2, s.u64x4_splat(SPLIT_M1));
        let x4 = s.i16x16_mul_lo(x1, s.u64x4_splat(SPLIT_M2));
        let m3 = s.u16x16_splat(SPLIT_M3);
        let x5 = s.v256_and(x3, m3);
        let x6 = s.v256_and(x4, m3);
        let x7 = s.u16x16_shr::<8>(x5);
        return s.v256_or(x6, x7);
    }

    {
        unreachable!()
    }
}

fn u32x8_blend_0x55<S: SIMD256>(s: S, a: V256, b: V256) -> V256 {
    if is_subtype!(S, AVX2) {
        return s.u32x8_blend::<0x55>(a, b);
    }
    if is_subtype!(S, SSE41) {
        return simd256_vop2(s, a, b, S::u32x4_blend::<0x5>);
    }
    {
        unreachable!()
    }
}

#[inline(always)]
pub fn merge_bits_simd<S: SIMD256>(s: S, x: V256) -> V256 {
    if is_subtype!(S, SSE41) {
        const MERGE_M1: u32 = u32::from_le_bytes([1 << 7, 1 << 2, 1 << 5, 1 << 0]);
        const MERGE_S1: V256 = V256::double_bytes([
            0x01, 0x00, 0x02, 0x04, 0x06, //
            0x09, 0x08, 0x0A, 0x0C, 0x0E, //
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
        ]);
        const MERGE_S2: V256 = V256::double_bytes([
            0x80, 0x03, 0x05, 0x07, 0x80, //
            0x80, 0x0B, 0x0D, 0x0F, 0x80, //
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
        ]);

        let x1 = s.i16x16_madd_sat(x, s.u32x8_splat(MERGE_M1));
        let x2 = s.u32x8_shl::<4>(x1);
        let x3 = u32x8_blend_0x55(s, x1, x2);
        let x4 = s.u8x16x2_swizzle(x3, MERGE_S1);
        let x5 = s.u8x16x2_swizzle(x3, MERGE_S2);
        return s.v256_or(x4, x5);
    }

    if is_subtype!(S, NEON | WASM128) {
        const MERGE_M1: u16 = u16::from_le_bytes([0x1f, 0x00]);
        const MERGE_M2: u64 = u16x4_to_u64([1 << 3, 1 << 1, 1 << 7, 1 << 5]);
        const MERGE_M3: u64 = u16x4_to_u64([1 << 6, 1 << 4, 1 << 2, 1 << 0]);

        const MERGE_S1: V256 = V256::double_bytes([
            0x00, 0x02, 0x05, 0x07, 0x06, 0x80, 0x80, 0x04, //
            0x08, 0x0A, 0x0D, 0x0F, 0x0E, 0x80, 0x80, 0x0C, //
        ]);
        const MERGE_S2: V256 = V256::double_bytes([
            0x01, 0x00, 0x02, 0x04, 0x06, 0x03, 0x80, 0x80, //
            0x09, 0x08, 0x0A, 0x0C, 0x0E, 0x0B, 0x80, 0x80, //
        ]);
        const MERGE_S3: V256 = V256::double_bytes([
            0x00, 0x01, 0x02, 0x03, 0x04, //
            0x08, 0x09, 0x0A, 0x0B, 0x0C, //
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
        ]);
        const MERGE_S4: V256 = V256::double_bytes([
            0x80, 0x05, 0x80, 0x07, 0x80, //
            0x80, 0x0D, 0x80, 0x0F, 0x80, //
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
        ]);

        let x1 = s.v256_and(x, s.u16x16_splat(MERGE_M1));
        let x2 = s.i16x16_mul_lo(x1, s.u64x4_splat(MERGE_M2));
        let x3 = s.u16x16_shr::<8>(x);
        let x4 = s.i16x16_mul_lo(x3, s.u64x4_splat(MERGE_M3));
        let x5 = s.u8x16x2_swizzle(x2, MERGE_S1);
        let x6 = s.u8x16x2_swizzle(x4, MERGE_S2);
        let x7 = s.v256_or(x5, x6);
        let x8 = s.u8x16x2_swizzle(x7, MERGE_S3);
        let x9 = s.u8x16x2_swizzle(x7, MERGE_S4);
        return s.v256_or(x8, x9);
    }

    {
        unreachable!()
    }
}

#[inline(always)]
pub fn encode_values_u8x32<S: SIMD256>(s: S, x: V256, lut0: V256, lut1: V256) -> V256 {
    if is_subtype!(S, SSE41) {
        let x1 = s.u8x16x2_swizzle(lut0, x);
        let x2 = s.u8x16x2_swizzle(lut1, x);
        let x3 = s.u8x32_lt(s.u8x32_splat(0x0f), x);
        return s.u8x32_blendv(x1, x2, x3);
    }
    if is_subtype!(S, NEON) && cfg!(target_arch = "aarch64") {
        let lo = lut0.to_v128x2().0;
        let hi = lut1.to_v128x2().1;
        let lut = V256::from_v128x2((lo, hi));
        return s.u8x32_swizzle(lut, x);
    }
    if is_subtype!(S, NEON | WASM128) {
        let x1 = s.u8x16x2_swizzle(lut0, x);
        let x2 = s.u8x16x2_swizzle(lut1, x);
        let x3 = s.u8x32_lt(s.u8x32_splat(0x0f), x);
        return s.v256_bsl(x3, x2, x1);
    }
    {
        unreachable!()
    }
}
