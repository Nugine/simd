#![allow(missing_docs)]

use crate::sa_hex::unhex;
use crate::{AsciiCase, Error, Hex, ERROR};

#[inline(always)]
const fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

const UPPER_TABLE: &[u8; 16] = b"0123456789ABCDEF";
const LOWER_TABLE: &[u8; 16] = b"0123456789abcdef";

#[inline]
pub const fn parse(src: &[u8]) -> Result<[u8; 16], Error> {
    match (src.len(), src) {
        (32, _) => parse_simple(src),
        (36, s)
        | (38, [b'{', s @ .., b'}'])
        | (45, [b'u', b'r', b'n', b':', b'u', b'u', b'i', b'd', b':', s @ ..]) => {
            parse_hyphenated(s)
        }
        _ => Err(ERROR),
    }
}

#[inline]
pub const fn parse_simple(src: &[u8]) -> Result<[u8; 16], Error> {
    if src.len() != 32 {
        return Err(ERROR);
    }

    let mut buf: [u8; 16] = [0; 16];
    let mut i = 0;
    while i < 16 {
        let h1 = unhex(src[i * 2]);
        let h2 = unhex(src[i * 2 + 1]);
        if h1 | h2 == 0xff {
            return Err(ERROR);
        }
        buf[i] = shl4(h1) | h2;
        i += 1;
    }
    Ok(buf)
}

#[inline]
pub const fn parse_hyphenated(src: &[u8]) -> Result<[u8; 16], Error> {
    if src.len() != 36 {
        return Err(ERROR);
    }

    match [src[8], src[13], src[18], src[23]] {
        [b'-', b'-', b'-', b'-'] => {}
        _ => return Err(ERROR),
    }

    let positions: [u8; 8] = [0, 4, 9, 14, 19, 24, 28, 32];
    let mut buf: [u8; 16] = [0; 16];
    let mut j = 0;
    while j < 8 {
        let i = positions[j];
        let h1 = unhex(src[i as usize]);
        let h2 = unhex(src[(i + 1) as usize]);
        let h3 = unhex(src[(i + 2) as usize]);
        let h4 = unhex(src[(i + 3) as usize]);
        if h1 | h2 | h3 | h4 == 0xff {
            return Err(ERROR);
        }
        buf[j * 2] = shl4(h1) | h2;
        buf[j * 2 + 1] = shl4(h3) | h4;
        j += 1;
    }

    Ok(buf)
}

const fn char_lut(case: AsciiCase) -> &'static [u8; 16] {
    match case {
        AsciiCase::Lower => LOWER_TABLE,
        AsciiCase::Upper => UPPER_TABLE,
    }
}

#[inline]
pub const fn format_simple(src: &[u8; 16], case: AsciiCase) -> Hex<32> {
    let lut = char_lut(case);
    let mut dst = [0; 32];
    let mut i = 0;
    while i < 16 {
        let x = src[i];
        dst[i * 2] = lut[(x >> 4) as usize];
        dst[i * 2 + 1] = lut[(x & 0x0f) as usize];
        i += 1;
    }
    unsafe { Hex::new_unchecked(dst) }
}

#[inline]
pub const fn format_hyphenated(src: &[u8; 16], case: AsciiCase) -> Hex<36> {
    let lut = char_lut(case);
    let groups = [(0, 8), (9, 13), (14, 18), (19, 23), (24, 36)];
    let mut dst = [0; 36];

    let mut group_idx = 0;
    let mut i = 0;
    while group_idx < 5 {
        let (start, end) = groups[group_idx];
        let mut j = start;
        while j < end {
            let x = src[i];
            i += 1;

            dst[j] = lut[(x >> 4) as usize];
            dst[j + 1] = lut[(x & 0x0f) as usize];
            j += 2;
        }
        if group_idx < 4 {
            dst[end] = b'-';
        }
        group_idx += 1;
    }
    unsafe { Hex::new_unchecked(dst) }
}

#[test]
fn test_parse() {
    super::tests::test_parse_ok(|s| parse(s.as_bytes()));
    super::tests::test_parse_err(|s| parse(s.as_bytes()));
}

#[test]
fn test_format() {
    super::tests::test_format_simple(format_simple);
    super::tests::test_format_hypenated(format_hyphenated);
}
