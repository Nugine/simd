use crate::AsciiCase;

use vsimd::tools::{read, slice};
use vsimd::SIMD256;

const fn full_table(table: &[u8; 16]) -> [u16; 256] {
    let mut buf = [0; 256];
    let mut i = 0;
    while i < 256 {
        let hi = table[i >> 4];
        let lo = table[i & 0xf];
        buf[i] = u16::from_ne_bytes([hi, lo]);
        i += 1;
    }
    buf
}

const UPPER_TABLE: &[u8; 16] = b"0123456789ABCDEF";
const LOWER_TABLE: &[u8; 16] = b"0123456789abcdef";

const FULL_LOWER_TABLE: &[u16; 256] = &full_table(LOWER_TABLE);
const FULL_UPPER_TABLE: &[u16; 256] = &full_table(UPPER_TABLE);

#[inline(always)]
pub unsafe fn encode_fallback(src: &[u8], mut dst: *mut u8, case: AsciiCase) {
    let table = match case {
        AsciiCase::Lower => FULL_LOWER_TABLE.as_ptr(),
        AsciiCase::Upper => FULL_UPPER_TABLE.as_ptr(),
    };
    let (mut src, len) = (src.as_ptr(), src.len());
    let end = src.add(len);
    while src < end {
        let x = src.read();
        let y = read(table, x as usize);
        dst.cast::<u16>().write_unaligned(y);
        src = src.add(1);
        dst = dst.add(2);
    }
}

#[inline(always)]
pub unsafe fn encode_simd<S: SIMD256>(s: S, src: &[u8], mut dst: *mut u8, case: AsciiCase) {
    let lut = match case {
        AsciiCase::Lower => vsimd::hex::ENCODE_LOWER_LUT,
        AsciiCase::Upper => vsimd::hex::ENCODE_UPPER_LUT,
    };

    let (mut src, mut len) = (src.as_ptr(), src.len());

    while len >= 32 {
        let x = s.v256_load_unaligned(src);
        let (y1, y2) = vsimd::hex::encode_bytes32(s, x, lut);

        s.v256_store_unaligned(dst, y1);
        dst = dst.add(32);

        s.v256_store_unaligned(dst, y2);
        dst = dst.add(32);

        src = src.add(32);
        len -= 32;
    }

    if len >= 16 {
        let x = s.v128_load_unaligned(src);
        let y = vsimd::hex::encode_bytes16(s, x, lut);
        s.v256_store_unaligned(dst, y);
        dst = dst.add(32);
        src = src.add(16);
        len -= 16;
    }

    if len > 0 {
        encode_fallback(slice(src, len), dst, case);
    }
}
