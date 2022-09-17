use crate::alsw::{self, AlswLutX2};
use crate::mask::{mask8x16_all, mask8x32_all, u8x32_highbit_any};
use crate::{AVX2, NEON, SIMD128, SIMD256, SSE41, V128, V256, WASM128};

const fn parse_hex(x: u8) -> u8 {
    match x {
        b'0'..=b'9' => x - b'0',
        b'a'..=b'f' => x - b'a' + 10,
        b'A'..=b'F' => x - b'A' + 10,
        _ => 0xff,
    }
}

#[inline(always)]
#[must_use]
pub const fn unhex(x: u8) -> u8 {
    const UNHEX_TABLE: &[u8; 256] = &{
        let mut arr = [0; 256];
        let mut i = 0;
        while i < 256 {
            arr[i] = parse_hex(i as u8);
            i += 1;
        }
        arr
    };
    UNHEX_TABLE[x as usize]
}

#[inline(always)]
pub fn check_ascii16<S: SIMD128>(s: S, x: V128) -> bool {
    let x1 = s.u8x16_sub(x, s.u8x16_splat(0xb0));
    let x2 = s.v128_and(x1, s.u8x16_splat(0xdf));
    let x3 = s.u8x16_sub(x2, s.u8x16_splat(0x11));
    let x4 = s.i8x16_lt(x1, s.i8x16_splat(-118));
    let x5 = s.i8x16_lt(x3, s.i8x16_splat(-122));
    let x6 = s.v128_or(x4, x5);
    mask8x16_all(s, x6)
}

#[inline(always)]
pub fn check_ascii32<S: SIMD256>(s: S, x: V256) -> bool {
    let x1 = s.u8x32_sub(x, s.u8x32_splat(0xb0));
    let x2 = s.v256_and(x1, s.u8x32_splat(0xdf));
    let x3 = s.u8x32_sub(x2, s.u8x32_splat(0x11));
    let x4 = s.i8x32_lt(x1, s.i8x32_splat(-118));
    let x5 = s.i8x32_lt(x3, s.i8x32_splat(-122));
    let x6 = s.v256_or(x4, x5);
    mask8x32_all(s, x6)
}

pub const ENCODE_UPPER_LUT: V256 = V256::double_bytes(*b"0123456789ABCDEF");
pub const ENCODE_LOWER_LUT: V256 = V256::double_bytes(*b"0123456789abcdef");

#[inline(always)]
pub fn encode_bytes16<S: SIMD256>(s: S, x: V128, lut: V256) -> V256 {
    let x = s.u16x16_from_u8x16(x);
    let hi = s.u16x16_shl::<8>(x);
    let lo = s.u16x16_shr::<4>(x);
    let values = s.v256_and(s.v256_or(hi, lo), s.u8x32_splat(0x0f));
    s.u8x16x2_swizzle(lut, values)
}

#[inline(always)]
pub fn encode_bytes32<S: SIMD256>(s: S, x: V256, lut: V256) -> (V256, V256) {
    let m = s.u8x32_splat(0x0f);
    let hi = s.v256_and(s.u16x16_shr::<4>(x), m);
    let lo = s.v256_and(x, m);

    let ac = s.u8x16x2_zip_lo(hi, lo);
    let bd = s.u8x16x2_zip_hi(hi, lo);

    let ab = s.v128x2_zip_lo(ac, bd);
    let cd = s.v128x2_zip_hi(ac, bd);

    let y1 = s.u8x16x2_swizzle(lut, ab);
    let y2 = s.u8x16x2_swizzle(lut, cd);

    (y1, y2)
}

struct HexAlsw;

impl HexAlsw {
    const fn decode(c: u8) -> u8 {
        parse_hex(c)
    }

    const fn check_hash(i: u8) -> u8 {
        match i {
            0 => 1,
            1..=6 => 1,
            7..=9 => 6,
            0xA..=0xF => 8,
            _ => unreachable!(),
        }
    }

    const fn decode_hash(i: u8) -> u8 {
        Self::check_hash(i)
    }
}

impl_alsw!(HexAlsw);

const HEX_ALSW_CHECK: AlswLutX2 = HexAlsw::check_lut_x2();
const HEX_ALSW_DECODE: AlswLutX2 = HexAlsw::decode_lut_x2();

const DECODE_UZP1: V256 = V256::double_bytes([
    0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, //
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
]);

const DECODE_UZP2: V256 = V256::double_bytes([
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
    0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, //
]);

#[inline(always)]
fn decode<S: SIMD256>(s: S, x: V256) -> (V256, V256) {
    let (c1, c2) = alsw::decode_ascii32(s, x, HEX_ALSW_CHECK, HEX_ALSW_DECODE);

    let b1 = s.u16x16_shl::<4>(c2);
    let b2 = s.u16x16_shr::<12>(b1);
    (s.v256_or(b1, b2), c1)
}

#[allow(clippy::result_unit_err)]
#[inline(always)]
pub fn decode_ascii32<S: SIMD256>(s: S, x: V256) -> Result<V128, ()> {
    let (y, is_invalid) = decode(s, x);

    let ans = if is_subtype!(S, SSE41 | WASM128) {
        let (a, b) = s.u8x16x2_swizzle(y, DECODE_UZP1).to_v128x2();
        s.u64x2_zip_lo(a, b)
    } else if is_subtype!(S, NEON) {
        let (a, b) = y.to_v128x2();
        s.u8x16_unzip_even(a, b)
    } else {
        unreachable!()
    };

    if u8x32_highbit_any(s, is_invalid) {
        Err(())
    } else {
        Ok(ans)
    }
}

#[allow(clippy::result_unit_err)]
#[inline(always)]
pub fn decode_ascii32x2<S: SIMD256>(s: S, x: (V256, V256)) -> Result<V256, ()> {
    let (y1, is_invalid1) = decode(s, x.0);
    let (y2, is_invalid2) = decode(s, x.1);
    let is_invalid = s.v256_or(is_invalid1, is_invalid2);

    let ans = if is_subtype!(S, AVX2) {
        let ab = s.u8x16x2_swizzle(y1, DECODE_UZP1);
        let cd = s.u8x16x2_swizzle(y2, DECODE_UZP2);
        let acbd = s.v256_or(ab, cd);
        s.u64x4_permute::<0b11011000>(acbd) // 0213
    } else if is_subtype!(S, SSE41 | WASM128) {
        let ab = s.u8x16x2_swizzle(y1, DECODE_UZP1);
        let cd = s.u8x16x2_swizzle(y2, DECODE_UZP1);
        s.u64x4_unzip_even(ab, cd)
    } else if is_subtype!(S, NEON) {
        s.u8x32_unzip_even(y1, y2)
    } else {
        unreachable!()
    };

    if u8x32_highbit_any(s, is_invalid) {
        Err(())
    } else {
        Ok(ans)
    }
}

#[cfg(test)]
mod algorithm {
    use super::*;

    #[ignore]
    #[test]
    fn check() {
        fn is_hex_v1(c: u8) -> bool {
            matches!(c, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')
        }

        fn is_hex_v2(c: u8) -> bool {
            let x1 = c.wrapping_sub(0x30);
            let x2 = (x1 & 0xdf).wrapping_sub(0x11);
            x1 < 10 || x2 < 6
        }

        fn is_hex_v3(c: u8) -> bool {
            let x1 = c.wrapping_sub(0xb0);
            let x2 = (x1 & 0xdf).wrapping_sub(0x11);
            ((x1 as i8) < -118) || ((x2 as i8) < -122)
        }

        for c in 0..=255_u8 {
            let (v1, v2, v3) = (is_hex_v1(c), is_hex_v2(c), is_hex_v3(c));
            assert_eq!(v1, v2);
            assert_eq!(v1, v3);
        }
    }

    #[ignore]
    #[test]
    fn hex_alsw() {
        HexAlsw::test_check();
        HexAlsw::test_decode();
    }
}
