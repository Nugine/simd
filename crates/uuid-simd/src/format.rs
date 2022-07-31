use crate::sa_ascii::AsciiCase;
use crate::sa_hex;
use crate::spec::SIMDExt;

use simd_abstraction::isa::{SimdLoad, SIMD256};
use simd_abstraction::scalar::Bytes32;
use simd_abstraction::tools::{read, write};

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
const fn char_lut_simd(case: AsciiCase) -> &'static Bytes32 {
    match case {
        AsciiCase::Lower => sa_hex::ENCODE_LOWER_LUT,
        AsciiCase::Upper => sa_hex::ENCODE_UPPER_LUT,
    }
}

#[inline]
pub unsafe fn format_simple_raw_fallback(src: *const u8, dst: *mut u8, case: AsciiCase) {
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
pub unsafe fn format_simple_raw_simd<S: SIMD256>(s: S, src: *const u8, dst: *mut u8, case: AsciiCase) {
    let lut = s.load(char_lut_simd(case));
    let a = s.v128_load_unaligned(src);
    let ans = sa_hex::encode_u8x16(s, a, lut);
    s.v256_store_unaligned(dst, ans);
}

#[inline]
pub unsafe fn format_hyphenated_raw_fallback(src: *const u8, dst: *mut u8, case: AsciiCase) {
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
pub unsafe fn format_hyphenated_raw_simd<S: SIMDExt>(s: S, src: *const u8, dst: *mut u8, case: AsciiCase) {
    const SWIZZLE: &Bytes32 = &Bytes32([
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, //
        0x80, 0x08, 0x09, 0x0a, 0x0b, 0x80, 0x0c, 0x0d, //
        0x80, 0x80, 0x80, 0x00, 0x01, 0x02, 0x03, 0x80, //
        0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, //
    ]);

    const DASH: &Bytes32 = &Bytes32([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x2d, 0x00, 0x00, 0x00, 0x00, 0x2d, 0x00, 0x00, //
        0x00, 0x00, 0x2d, 0x00, 0x00, 0x00, 0x00, 0x2d, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);

    let lut = s.load(char_lut_simd(case));
    let a = sa_hex::encode_u8x16(s, s.v128_load_unaligned(src), lut);

    let a1 = s.u8x16x2_swizzle(a, s.load(SWIZZLE));
    let a2 = s.v256_or(a1, s.load(DASH));
    s.v256_store_unaligned(dst, a2);

    let bytes_14_15 = s.i16x8_get_lane7(s.v256_get_low(a)) as u16;
    let bytes_28_31 = s.i32x4_get_lane3(s.v256_get_high(a)) as u32;
    core::ptr::write_unaligned(dst.add(16).cast(), bytes_14_15);
    core::ptr::write_unaligned(dst.add(32).cast(), bytes_28_31);
}
