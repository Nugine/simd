use crate::mask::{mask8x16_all, mask8x32_all};
use crate::{AVX2, NEON, SIMD128, SIMD256, SSE41, V128, V256, WASM128};

use core::ops::Not;

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

#[inline(always)]
fn split_hilo<S: SIMD256>(s: S, x: V256) -> (V256, V256) {
    let m = s.u8x32_splat(0x0f);
    let hi = s.v256_and(s.u16x16_shr::<4>(x), m);
    let lo = s.v256_and(x, m);
    (hi, lo)
}

#[inline(always)]
fn check_hilo<S: SIMD256>(s: S, (hi, lo): (V256, V256)) -> bool {
    //  '0'~'9'     0x3     0~9     0x0f
    //  'A'~'F'     0x4     1~6     0xf0
    //  'a'~'f'     0x6     1~6     0xf0
    //  ...                         0x00

    const HI_LUT: V256 = V256::double_bytes([
        0x00, 0x00, 0x00, 0x0f, 0xf0, 0x00, 0xf0, 0x00, // [3] = 0x0f, [4,6] = 0xf0
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // [others] = 0x00
    ]);
    const LI_LUT: V256 = V256::double_bytes([
        0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, // [0~9] = 0x?f, [1~6] = 0xf?
        0x0f, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // [others] = 0x00
    ]);

    let hi_flag = s.u8x16x2_swizzle(HI_LUT, hi);
    let lo_flag = s.u8x16x2_swizzle(LI_LUT, lo);
    let flag = s.v256_and(hi_flag, lo_flag);
    !s.u8x32_any_zero(flag)
}

#[inline(always)]
fn decode_hilo<S: SIMD256>(s: S, (hi, lo): (V256, V256)) -> V256 {
    //  '0'     0x30    +0
    //  'A'     0x41    +9
    //  'a'     0x61    +9

    const OFFSET_LUT: V256 = V256::double_bytes([
        0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x09, 0x00, // [4,6] = 9
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // [others] = 0
    ]);

    let offset = s.u8x16x2_swizzle(OFFSET_LUT, hi);
    let x = s.u8x32_add(lo, offset);
    // x:   {0000hhhh|0000llll}x16

    let x1 = s.u16x16_shl::<4>(x);
    // x1:  {hhhh0000|llll0000}x16

    let x2 = s.u16x16_shr::<12>(x1);
    // x2:  {0000llll|00000000}x16

    s.v256_or(x1, x2)
    //      {hhhhllll|llll0000}x16
}

const DECODE_UZP1: V256 = V256::double_bytes([
    0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, //
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
]);

const DECODE_UZP2: V256 = V256::double_bytes([
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
    0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, //
]);

#[allow(clippy::result_unit_err)]
#[inline(always)]
pub fn decode_ascii32<S: SIMD256>(s: S, x: V256) -> Result<V128, ()> {
    let t = split_hilo(s, x);

    if check_hilo(s, t).not() {
        return Err(());
    }

    let y = decode_hilo(s, t);

    if is_subtype!(S, SSE41 | WASM128) {
        let (a, b) = s.u8x16x2_swizzle(y, DECODE_UZP1).to_v128x2();
        return Ok(s.u64x2_zip_lo(a, b));
    }

    if is_subtype!(S, NEON) {
        let (a, b) = y.to_v128x2();
        return Ok(s.u8x16_unzip_even(a, b));
    }

    {
        unreachable!()
    }
}

#[allow(clippy::result_unit_err)]
#[inline(always)]
pub fn decode_ascii32x2<S: SIMD256>(s: S, x: (V256, V256)) -> Result<V256, ()> {
    let (t1, t2) = (split_hilo(s, x.0), split_hilo(s, x.1));

    if check_hilo(s, t1).not() || check_hilo(s, t2).not() {
        return Err(());
    }

    let (y1, y2) = (decode_hilo(s, t1), decode_hilo(s, t2));

    if is_subtype!(S, AVX2) {
        let ab = s.u8x16x2_swizzle(y1, DECODE_UZP1);
        let cd = s.u8x16x2_swizzle(y2, DECODE_UZP2);
        let acbd = s.v256_or(ab, cd);
        let abcd = s.u64x4_permute::<0b11011000>(acbd); // 0213
        return Ok(abcd);
    }

    if is_subtype!(S, SSE41 | WASM128) {
        let ab = s.u8x16x2_swizzle(y1, DECODE_UZP1);
        let cd = s.u8x16x2_swizzle(y2, DECODE_UZP1);
        return Ok(s.u64x4_unzip_even(ab, cd));
    }

    if is_subtype!(S, NEON) {
        return Ok(s.u8x32_unzip_even(y1, y2));
    }

    {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    #[ignore] // algorithm checker
    #[test]
    fn hex_check() {
        fn is_hex_v1(c: u8) -> bool {
            matches!(c, b'0'..=b'9'|b'a'..=b'f'|b'A'..=b'F')
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

        fn is_hex_v4(c: u8) -> bool {
            let hi_lut = &[
                0x00, 0x00, 0x00, 0x0f, 0xf0, 0x00, 0xf0, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
            ];
            let lo_lut = &[
                0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, //
                0x0f, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
            ];
            let (hi, lo) = (c >> 4, c & 0x0f);
            (hi_lut[hi as usize] & lo_lut[lo as usize]) != 0
        }

        for c in 0..=255_u8 {
            let (v1, v2, v3, v4) = (is_hex_v1(c), is_hex_v2(c), is_hex_v3(c), is_hex_v4(c));
            assert_eq!(v1, v2);
            assert_eq!(v1, v3);
            assert_eq!(v1, v4);
        }
    }
}
