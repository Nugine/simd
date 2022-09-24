use crate::alsw::{self, AlswLut};
use crate::isa::{AVX2, NEON, SSE41, WASM128};
use crate::mask::u8x32_highbit_any;
use crate::vector::{V128, V256};
use crate::SIMD256;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Base32,
    Base32Hex,
}

pub const BASE32_CHARSET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
pub const BASE32HEX_CHARSET: &[u8; 32] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

#[inline(always)]
const fn u16x4_to_u64(x: [u16; 4]) -> u64 {
    unsafe { core::mem::transmute(x) }
}

#[inline(always)]
fn split_bits<S: SIMD256>(s: S, x: V256) -> V256 {
    const SPLIT_SHUFFLE: V256 = V256::from_bytes([
        0x07, 0x06, 0x08, 0x07, 0x09, 0x08, 0x0A, 0x09, //
        0x0C, 0x0B, 0x0D, 0x0C, 0x0E, 0x0D, 0x0F, 0x0E, //
        0x01, 0x00, 0x02, 0x01, 0x03, 0x02, 0x04, 0x03, //
        0x06, 0x05, 0x07, 0x06, 0x08, 0x07, 0x09, 0x08, //
    ]);

    if is_subtype!(S, SSE41) {
        const SPLIT_M1: u64 = u16x4_to_u64([1 << 5, 1 << 7, 1 << 9, 1 << 11]);
        const SPLIT_M2: u64 = u16x4_to_u64([1 << 2, 1 << 4, 1 << 6, 1 << 8]);

        let x1 = s.u8x16x2_swizzle(x, SPLIT_SHUFFLE);
        let x2 = s.u16x16_mul_hi(x1, s.u64x4_splat(SPLIT_M1));
        let x3 = s.i16x16_mul_lo(x1, s.u64x4_splat(SPLIT_M2));
        let x4 = s.v256_and(x2, s.u16x16_splat(u16::from_le_bytes([0x1f, 0x00])));
        let x5 = s.v256_and(x3, s.u16x16_splat(u16::from_le_bytes([0x00, 0x1f])));
        return s.v256_or(x4, x5);
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

    unreachable!()
}

#[inline(always)]
fn u32x8_blend_0x55<S: SIMD256>(s: S, a: V256, b: V256) -> V256 {
    if is_subtype!(S, AVX2) {
        return s.u32x8_blend::<0x55>(a, b);
    }
    if is_subtype!(S, SSE41) {
        return simd256_vop!(s, S::u32x4_blend::<0x5>, a, b);
    }
    unreachable!()
}

#[inline(always)]
fn merge_bits<S: SIMD256>(s: S, x: V256) -> V256 {
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

        let x1 = s.i16x16_maddubs(s.u32x8_splat(MERGE_M1), x);
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

    unreachable!()
}

#[derive(Debug, Clone, Copy)]
pub struct EncodingLutX2 {
    low: V256,
    high: V256,
    full: V256,
}

impl EncodingLutX2 {
    const fn new(charset: &[u8; 32]) -> Self {
        let full = V256::from_bytes(*charset);
        let charset: &[[u8; 16]; 2] = unsafe { core::mem::transmute(charset) };
        let low = V256::double_bytes(charset[0]);
        let high = V256::double_bytes(charset[1]);
        Self { low, high, full }
    }
}

pub const BASE32_ENCODING_LUT: EncodingLutX2 = EncodingLutX2::new(BASE32_CHARSET);
pub const BASE32HEX_ENCODING_LUT: EncodingLutX2 = EncodingLutX2::new(BASE32HEX_CHARSET);

#[inline(always)]
fn encode_values<S: SIMD256>(s: S, x: V256, lut: EncodingLutX2) -> V256 {
    if is_subtype!(S, SSE41) {
        let x1 = s.u8x16x2_swizzle(lut.low, x);
        let x2 = s.u8x16x2_swizzle(lut.high, x);
        let x3 = s.u8x32_lt(s.u8x32_splat(0x0f), x);
        return s.u8x32_blendv(x1, x2, x3);
    }
    if is_subtype!(S, NEON) && cfg!(target_arch = "aarch64") {
        return s.u8x32_swizzle(lut.full, x);
    }
    if is_subtype!(S, NEON | WASM128) {
        let m = s.u8x32_splat(0x0f);
        let x1 = s.v256_and(x, m);
        let x2 = s.u8x16x2_swizzle(lut.low, x1);
        let x3 = s.u8x16x2_swizzle(lut.high, x1);
        let x4 = s.u8x32_lt(m, x);
        return s.v256_bsl(x4, x3, x2);
    }
    unreachable!()
}

#[inline(always)]
pub fn encode_bytes20<S: SIMD256>(s: S, x: V256, lut: EncodingLutX2) -> V256 {
    // x: {????|??AA|AAAB|BBBB|CCCC|CDDD|DD??|????}

    let values = split_bits(s, x);
    // values: {000xyyyy}x32

    encode_values(s, values, lut)
    // {{ascii}}x32
}

struct Base32Alsw;

impl Base32Alsw {
    const fn decode(c: u8) -> u8 {
        match c {
            b'A'..=b'Z' => c - b'A',
            b'2'..=b'7' => c - b'2' + 26,
            _ => 0xff,
        }
    }

    const fn check_hash(i: u8) -> u8 {
        match i {
            0x0 => 1,
            0x1 => 1,
            0x2..=0x7 => 6,
            0x8..=0xA => 1,
            0xB..=0xF => 7,
            _ => unreachable!(),
        }
    }

    const fn decode_hash(i: u8) -> u8 {
        Self::check_hash(i)
    }
}

impl_alsw!(Base32Alsw);

struct Base32HexAlsw;

impl Base32HexAlsw {
    const fn decode(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'V' => c - b'A' + 10,
            _ => 0xff,
        }
    }

    const fn check_hash(i: u8) -> u8 {
        match i {
            0 => 1,
            1..=6 => 1,
            7..=9 => 7,
            0xA..=0xF => 2,
            _ => unreachable!(),
        }
    }

    const fn decode_hash(i: u8) -> u8 {
        Self::check_hash(i)
    }
}

impl_alsw!(Base32HexAlsw);

pub const BASE32_ALSW_CHECK_X2: AlswLut<V256> = Base32Alsw::check_lut().x2();
pub const BASE32_ALSW_DECODE_X2: AlswLut<V256> = Base32Alsw::decode_lut().x2();

pub const BASE32HEX_ALSW_CHECK_X2: AlswLut<V256> = Base32HexAlsw::check_lut().x2();
pub const BASE32HEX_ALSW_DECODE_X2: AlswLut<V256> = Base32HexAlsw::decode_lut().x2();

#[inline(always)]
pub fn check_ascii32<S: SIMD256>(s: S, x: V256, check: AlswLut<V256>) -> bool {
    crate::alsw::check_ascii_xn(s, x, check)
}

#[allow(clippy::result_unit_err)]
#[inline(always)]
pub fn decode_ascii32<S: SIMD256>(s: S, x: V256, check: AlswLut<V256>, decode: AlswLut<V256>) -> Result<V256, ()> {
    let (c1, c2) = alsw::decode_ascii_xn(s, x, check, decode);

    let y = merge_bits(s, c2);

    if u8x32_highbit_any(s, c1) {
        Err(())
    } else {
        Ok(y)
    }
}

#[cfg(test)]
mod algorithm {
    use super::*;

    #[test]
    #[ignore]
    fn base32_alsw() {
        Base32Alsw::test_check();
        Base32Alsw::test_decode();
    }

    #[test]
    #[ignore]
    fn base32hex_alsw() {
        Base32HexAlsw::test_check();
        Base32HexAlsw::test_decode();
    }
}
