use crate::spec::*;

use vsimd::ascii::AsciiCase;
use vsimd::tools::{read, write};
use vsimd::{SIMD256, V256};

#[inline(always)]
const fn char_lut_fallback(case: AsciiCase) -> &'static [u8; 16] {
    const LOWER_TABLE: &[u8; 16] = b"0123456789abcdef";
    const UPPER_TABLE: &[u8; 16] = b"0123456789ABCDEF";

    match case {
        AsciiCase::Lower => LOWER_TABLE,
        AsciiCase::Upper => UPPER_TABLE,
    }
}

#[inline(always)]
const fn char_lut_simd(case: AsciiCase) -> V256 {
    match case {
        AsciiCase::Lower => vsimd::hex::ENCODE_LOWER_LUT,
        AsciiCase::Upper => vsimd::hex::ENCODE_UPPER_LUT,
    }
}

#[inline]
pub unsafe fn format_simple_fallback(src: *const u8, dst: *mut u8, case: AsciiCase) {
    let lut = char_lut_fallback(case).as_ptr();
    for i in 0..16 {
        let x = read(src, i);
        let hi = read(lut, (x >> 4) as usize);
        let lo = read(lut, (x & 0x0f) as usize);
        write(dst, i * 2, hi);
        write(dst, i * 2 + 1, lo);
    }
}

#[inline]
pub unsafe fn format_simple_simd<S: SIMD256>(s: S, src: *const u8, dst: *mut u8, case: AsciiCase) {
    let lut = char_lut_simd(case);
    let x = s.v128_load_unaligned(src);
    let y = vsimd::hex::encode_bytes16(s, x, lut);
    s.v256_store_unaligned(dst, y);
}

#[inline]
pub unsafe fn format_hyphenated_fallback(src: *const u8, dst: *mut u8, case: AsciiCase) {
    let lut = char_lut_fallback(case).as_ptr();
    let groups = [(0, 8), (9, 13), (14, 18), (19, 23), (24, 36)];

    let mut i = 0;
    for (g, (start, end)) in groups.iter().copied().enumerate() {
        for j in (start..end).step_by(2) {
            let x = read(src, i);
            i += 1;
            let hi = read(lut, (x >> 4) as usize);
            let lo = read(lut, (x & 0x0f) as usize);
            write(dst, j, hi);
            write(dst, j + 1, lo);
        }
        if g < 4 {
            write(dst, end, b'-');
        }
    }
}

#[inline]
pub unsafe fn format_hyphenated_simd<S: SIMD256>(s: S, src: *const u8, dst: *mut u8, case: AsciiCase) {
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

    let lut = char_lut_simd(case);
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
