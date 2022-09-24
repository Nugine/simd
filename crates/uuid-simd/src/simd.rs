use crate::spec::*;
use crate::Error;

use vsimd::ascii::AsciiCase;
use vsimd::tools::read;
use vsimd::vector::V256;
use vsimd::SIMD256;

#[inline(always)]
pub unsafe fn parse_simple<S: SIMD256>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let x = s.v256_load_unaligned(src);
    let y = vsimd::hex::decode_ascii32(s, x).map_err(|()| Error::new())?;
    s.v128_store_unaligned(dst, y);
    Ok(())
}

#[inline(always)]
pub unsafe fn parse_hyphenated<S: SIMD256>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    match [read(src, 8), read(src, 13), read(src, 18), read(src, 23)] {
        [b'-', b'-', b'-', b'-'] => {}
        _ => return Err(Error::new()),
    }

    const SWIZZLE: V256 = V256::from_bytes([
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, //
        0x09, 0x0a, 0x0b, 0x0c, 0x0e, 0x0f, 0x80, 0x80, //
        0x03, 0x04, 0x05, 0x06, 0x08, 0x09, 0x0a, 0x0b, //
        0x0c, 0x0d, 0x0e, 0x0f, 0x80, 0x80, 0x80, 0x80, //
    ]);

    let a0 = s.v256_load_unaligned(src);
    let a1 = s.u8x16x2_swizzle(a0, SWIZZLE);

    let a2 = i16x16_set_lane7(s, a1, src.add(16).cast::<i16>().read_unaligned());
    let a3 = i32x8_set_lane7(s, a2, src.add(32).cast::<i32>().read_unaligned());

    let ans = vsimd::hex::decode_ascii32(s, a3).map_err(|()| Error::new())?;
    s.v128_store_unaligned(dst, ans);

    Ok(())
}

#[inline(always)]
const fn char_lut(case: AsciiCase) -> V256 {
    match case {
        AsciiCase::Lower => vsimd::hex::ENCODE_LOWER_LUT,
        AsciiCase::Upper => vsimd::hex::ENCODE_UPPER_LUT,
    }
}

#[inline(always)]
pub unsafe fn format_simple<S: SIMD256>(s: S, src: *const u8, dst: *mut u8, case: AsciiCase) {
    let lut = char_lut(case);
    let x = s.v128_load_unaligned(src);
    let y = vsimd::hex::encode_bytes16(s, x, lut);
    s.v256_store_unaligned(dst, y);
}

#[inline(always)]
pub unsafe fn format_hyphenated<S: SIMD256>(s: S, src: *const u8, dst: *mut u8, case: AsciiCase) {
    const SWIZZLE: V256 = V256::from_bytes([
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, //
        0x80, 0x08, 0x09, 0x0a, 0x0b, 0x80, 0x0c, 0x0d, //
        0x80, 0x80, 0x80, 0x00, 0x01, 0x02, 0x03, 0x80, //
        0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, //
    ]);

    const DASH: V256 = V256::from_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x2d, 0x00, 0x00, 0x00, 0x00, 0x2d, 0x00, 0x00, //
        0x00, 0x00, 0x2d, 0x00, 0x00, 0x00, 0x00, 0x2d, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);

    let lut = char_lut(case);
    let a = vsimd::hex::encode_bytes16(s, s.v128_load_unaligned(src), lut);

    let a1 = s.u8x16x2_swizzle(a, SWIZZLE);
    let a2 = s.v256_or(a1, DASH);
    s.v256_store_unaligned(dst, a2);

    let a = a.to_v128x2();
    let bytes_14_15 = i16x8_get_lane7(s, a.0) as u16;
    let bytes_28_31 = i32x4_get_lane3(s, a.1) as u32;
    core::ptr::write_unaligned(dst.add(16).cast(), bytes_14_15);
    core::ptr::write_unaligned(dst.add(32).cast(), bytes_28_31);
}
