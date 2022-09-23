use crate::alsw::{self, AlswLut};
use crate::isa::{NEON, SSE41, WASM128};
use crate::mask::u8x32_highbit_any;
use crate::vector::{V128, V256};
use crate::{Scalable, SIMD128, SIMD256};

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Standard,
    UrlSafe,
}

pub const STANDARD_CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
pub const URL_SAFE_CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

const SPLIT_SHUFFLE: V256 = V256::from_bytes([
    0x05, 0x04, 0x06, 0x05, 0x08, 0x07, 0x09, 0x08, //
    0x0b, 0x0a, 0x0c, 0x0b, 0x0e, 0x0d, 0x0f, 0x0e, //
    0x01, 0x00, 0x02, 0x01, 0x04, 0x03, 0x05, 0x04, //
    0x07, 0x06, 0x08, 0x07, 0x0a, 0x09, 0x0b, 0x0a, //
]);

#[inline(always)]
fn split_bits_x2<S: SIMD256>(s: S, x: V256) -> V256 {
    // x: {????|AAAB|BBCC|CDDD|EEEF|FFGG|GHHH|????}

    let x0 = s.u8x16x2_swizzle(x, SPLIT_SHUFFLE);
    // x0: {bbbbcccc|aaaaaabb|ccdddddd|bbbbcccc} x8 (1021)

    if is_subtype!(S, SSE41) {
        let m1 = s.u32x8_splat(u32::from_le_bytes([0x00, 0xfc, 0xc0, 0x0f]));
        let x1 = s.v256_and(x0, m1);
        // x1: {00000000|aaaaaa00|cc000000|0000cccc} x8

        let m2 = s.u32x8_splat(u32::from_le_bytes([0xf0, 0x03, 0x3f, 0x00]));
        let x2 = s.v256_and(x0, m2);
        // x2: {bbbb0000|000000bb|00dddddd|00000000} x8

        let m3 = s.u32x8_splat(u32::from_le_bytes([0x40, 0x00, 0x00, 0x04]));
        let x3 = s.u16x16_mul_hi(x1, m3);
        // x3: {00aaaaaa|00000000|00cccccc|00000000} x8

        let m4 = s.u32x8_splat(u32::from_le_bytes([0x10, 0x00, 0x00, 0x01]));
        let x4 = s.i16x16_mul_lo(x2, m4);
        // x4: {00000000|00bbbbbb|00000000|00dddddd} x8

        return s.v256_or(x3, x4);
        // {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8
    }

    if is_subtype!(S, NEON | WASM128) {
        let m1 = s.u32x8_splat(u32::from_le_bytes([0x00, 0xfc, 0x00, 0x00]));
        let x1 = s.u16x16_shr::<10>(s.v256_and(x0, m1));
        // x1: {00aaaaaa|000000000|00000000|00000000} x8

        let m2 = s.u32x8_splat(u32::from_le_bytes([0xf0, 0x03, 0x00, 0x00]));
        let x2 = s.u16x16_shl::<4>(s.v256_and(x0, m2));
        // x2: {00000000|00bbbbbb|00000000|00000000} x8

        let m3 = s.u32x8_splat(u32::from_le_bytes([0x00, 0x00, 0xc0, 0x0f]));
        let x3 = s.u16x16_shr::<6>(s.v256_and(x0, m3));
        // x3: {00000000|00000000|00cccccc|00000000} x8

        let m4 = s.u32x8_splat(u32::from_le_bytes([0x00, 0x00, 0x3f, 0x00]));
        let x4 = s.u16x16_shl::<8>(s.v256_and(x0, m4));
        // x4: {00000000|00000000|00000000|00dddddd} x8

        return s.v256_or(s.v256_or(x1, x2), s.v256_or(x3, x4));
        // {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8
    }

    unreachable!()
}

#[inline(always)]
fn split_bits_x1<S: SIMD128>(s: S, x: V128) -> V128 {
    // x: {AAAB|BBCC|CDDD|????}

    const SHUFFLE: V128 = SPLIT_SHUFFLE.to_v128x2().1;
    let x0 = s.u8x16_swizzle(x, SHUFFLE);
    // x0: {bbbbcccc|aaaaaabb|ccdddddd|bbbbcccc} x8 (1021)

    if is_subtype!(S, SSE41) {
        let m1 = s.u32x4_splat(u32::from_le_bytes([0x00, 0xfc, 0xc0, 0x0f]));
        let x1 = s.v128_and(x0, m1);

        let m2 = s.u32x4_splat(u32::from_le_bytes([0xf0, 0x03, 0x3f, 0x00]));
        let x2 = s.v128_and(x0, m2);

        let m3 = s.u32x4_splat(u32::from_le_bytes([0x40, 0x00, 0x00, 0x04]));
        let x3 = s.u16x8_mul_hi(x1, m3);

        let m4 = s.u32x4_splat(u32::from_le_bytes([0x10, 0x00, 0x00, 0x01]));
        let x4 = s.i16x8_mul_lo(x2, m4);

        return s.v128_or(x3, x4);
    }

    if is_subtype!(S, NEON | WASM128) {
        let m1 = s.u32x4_splat(u32::from_le_bytes([0x00, 0xfc, 0x00, 0x00]));
        let x1 = s.u16x8_shr::<10>(s.v128_and(x0, m1));

        let m2 = s.u32x4_splat(u32::from_le_bytes([0xf0, 0x03, 0x00, 0x00]));
        let x2 = s.u16x8_shl::<4>(s.v128_and(x0, m2));

        let m3 = s.u32x4_splat(u32::from_le_bytes([0x00, 0x00, 0xc0, 0x0f]));
        let x3 = s.u16x8_shr::<6>(s.v128_and(x0, m3));

        let m4 = s.u32x4_splat(u32::from_le_bytes([0x00, 0x00, 0x3f, 0x00]));
        let x4 = s.u16x8_shl::<8>(s.v128_and(x0, m4));

        return s.v128_or(s.v128_or(x1, x2), s.v128_or(x3, x4));
    }

    unreachable!()
}

#[inline(always)]
fn merge_bits_x2<S: SIMD256>(s: S, x: V256) -> V256 {
    // x : {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8

    let y = if is_subtype!(S, SSE41) {
        let m1 = s.u16x16_splat(u16::from_le_bytes([0x40, 0x01]));
        let x1 = s.i16x16_maddubs(x, m1);
        // x1: {aabbbbbb|0000aaaa|ccdddddd|0000cccc} x8

        let m2 = s.u32x8_splat(u32::from_le_bytes([0x00, 0x10, 0x01, 0x00]));
        s.i16x16_madd(x1, m2)
        // {ccdddddd|bbbbcccc|aaaaaabb|00000000} x8
    } else if is_subtype!(S, NEON | WASM128) {
        let m1 = s.u32x8_splat(u32::from_le_bytes([0x3f, 0x00, 0x3f, 0x00]));
        let x1 = s.v256_and(x, m1);
        // x1: {00aaaaaa|00000000|00cccccc|00000000} x8

        let m2 = s.u32x8_splat(u32::from_le_bytes([0x00, 0x3f, 0x00, 0x3f]));
        let x2 = s.v256_and(x, m2);
        // x2: {00000000|00bbbbbb|00000000|00dddddd} x8

        let x3 = s.v256_or(s.u32x8_shl::<18>(x1), s.u32x8_shr::<10>(x1));
        // x3: {cc000000|0000cccc|aaaaaa00|00000000} x8

        let x4 = s.v256_or(s.u32x8_shl::<4>(x2), s.u32x8_shr::<24>(x2));
        // x4: {00dddddd|bbbb0000|000000bb|dddd0000}

        let mask = s.u32x8_splat(u32::from_le_bytes([0xff, 0xff, 0xff, 0x00]));
        s.v256_and(s.v256_or(x3, x4), mask)
        // {ccdddddd|bbbbcccc|aaaaaabb|00000000} x8
    } else {
        unreachable!()
    };

    const SHUFFLE: V256 = V256::double_bytes([
        0x02, 0x01, 0x00, 0x06, 0x05, 0x04, 0x0a, 0x09, //
        0x08, 0x0e, 0x0d, 0x0c, 0x80, 0x80, 0x80, 0x80, //
    ]);
    s.u8x16x2_swizzle(y, SHUFFLE)
    // {AAAB|BBCC|CDDD|0000|EEEF|FFGG|GHHH|0000}
}

const fn encoding_shift(charset: &'static [u8; 64]) -> V128 {
    // 0~25     'A'   [13]
    // 26~51    'a'   [0]
    // 52~61    '0'   [1~10]
    // 62       c62   [11]
    // 63       c63   [12]

    let mut lut = [0x80; 16];
    lut[13] = b'A';
    lut[0] = b'a' - 26;
    let mut i = 1;
    while i <= 10 {
        lut[i] = b'0'.wrapping_sub(52);
        i += 1;
    }
    lut[11] = charset[62].wrapping_sub(62);
    lut[12] = charset[63].wrapping_sub(63);
    V128::from_bytes(lut)
}

pub const STANDARD_ENCODING_SHIFT: V128 = encoding_shift(STANDARD_CHARSET);
pub const URL_SAFE_ENCODING_SHIFT: V128 = encoding_shift(URL_SAFE_CHARSET);

pub const STANDARD_ENCODING_SHIFT_X2: V256 = STANDARD_ENCODING_SHIFT.x2();
pub const URL_SAFE_ENCODING_SHIFT_X2: V256 = URL_SAFE_ENCODING_SHIFT.x2();

fn encode_values<S: Scalable<V>, V: Copy>(s: S, x: V, shift_lut: V) -> V {
    // x: {00aaaaaa|00bbbbbb|00cccccc|00dddddd} xn

    let x1 = s.u8xn_sub_sat(x, s.u8xn_splat(51));
    // 0~25    => 0
    // 26~51   => 0
    // 52~61   => 1~10
    // 62      => 11
    // 63      => 12

    let x2 = s.i8xn_lt(x, s.u8xn_splat(26));
    let x3 = s.and(x2, s.u8xn_splat(13));
    let x4 = s.or(x1, x3);
    // 0~25    => 0xff  => 13
    // 26~51   => 0     => 0
    // 52~61   => 0     => 1~10
    // 62      => 0     => 11
    // 63      => 0     => 12

    let shift = s.u8x16xn_swizzle(shift_lut, x4);
    s.u8xn_add(x, shift)
    // {{ascii}} xn
}

#[inline(always)]
pub fn encode_bytes24<S: SIMD256>(s: S, x: V256, shift_lut: V256) -> V256 {
    // x: {????|AAAB|BBCC|CDDD|EEEF|FFGG|GHHH|????}

    let values = split_bits_x2(s, x);
    // values: {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8

    encode_values(s, values, shift_lut)
    // {{ascii}} x32
}

#[inline(always)]
pub fn encode_bytes12<S: SIMD256>(s: S, x: V128, shift_lut: V128) -> V128 {
    // x: {AAAB|BBCC|CDDD|????}

    let values = split_bits_x1(s, x);
    // values: {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x4

    encode_values(s, values, shift_lut)
    // {{ascii}} x16
}

struct StandardAlsw;

impl StandardAlsw {
    const fn decode(c: u8) -> u8 {
        match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => 0xff,
        }
    }

    const fn check_hash(i: u8) -> u8 {
        match i {
            0 => 5,
            1..=9 => 2,
            0xA => 4,
            0xB => 6,
            0xC..=0xE => 8,
            0xF => 6,
            _ => unreachable!(),
        }
    }

    const fn decode_hash(i: u8) -> u8 {
        match i {
            0xB => 0x07,
            0xF => 0x08,
            _ => 0x01,
        }
    }
}

impl_alsw!(StandardAlsw);

struct UrlSafeAlsw;

impl UrlSafeAlsw {
    const fn decode(c: u8) -> u8 {
        match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => 0xff,
        }
    }

    const fn check_hash(i: u8) -> u8 {
        match i {
            0 => 7,
            1..=9 => 2,
            0xA => 4,
            0xB | 0xC => 6,
            0xD => 8,
            0xE => 6,
            0xF => 6,
            _ => unreachable!(),
        }
    }

    const fn decode_hash(i: u8) -> u8 {
        match i {
            0xD => 0x01,
            0xF => 0x05,
            _ => 0x01,
        }
    }
}

impl_alsw!(UrlSafeAlsw);

pub const STANDARD_ALSW_CHECK_X2: AlswLut<V256> = StandardAlsw::check_lut().x2();
pub const STANDARD_ALSW_DECODE_X2: AlswLut<V256> = StandardAlsw::decode_lut().x2();

pub const URL_SAFE_ALSW_CHECK_X2: AlswLut<V256> = UrlSafeAlsw::check_lut().x2();
pub const URL_SAFE_ALSW_DECODE_X2: AlswLut<V256> = UrlSafeAlsw::decode_lut().x2();

#[inline(always)]
pub fn check_ascii32<S: SIMD256>(s: S, x: V256, check: AlswLut<V256>) -> bool {
    alsw::check_ascii_xn(s, x, check)
}

#[allow(clippy::result_unit_err)]
#[inline(always)]
pub fn decode_ascii32<S: SIMD256>(s: S, x: V256, check: AlswLut<V256>, decode: AlswLut<V256>) -> Result<V256, ()> {
    let (c1, c2) = alsw::decode_ascii_xn(s, x, check, decode);

    let y = merge_bits_x2(s, c2);

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
    fn standard_alsw() {
        StandardAlsw::test_check();
        StandardAlsw::test_decode();
    }

    #[test]
    #[ignore]
    fn url_safe_alsw() {
        UrlSafeAlsw::test_check();
        UrlSafeAlsw::test_decode();
    }
}
