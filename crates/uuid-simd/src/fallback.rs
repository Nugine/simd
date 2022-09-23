use crate::Error;

use vsimd::ascii::AsciiCase;
use vsimd::hex::unhex;
use vsimd::tools::{read, write};

#[inline(always)]
const fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

#[inline]
pub unsafe fn parse_simple(src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let mut flag = 0;
    for i in 0..16 {
        let h1 = unhex(read(src, i * 2));
        let h2 = unhex(read(src, i * 2 + 1));
        flag |= h1 | h2;
        write(dst, i, shl4(h1) | h2);
    }
    ensure!(flag != 0xff);
    Ok(())
}

#[inline]
pub unsafe fn parse_hyphenated(src: *const u8, dst: *mut u8) -> Result<(), Error> {
    match [read(src, 8), read(src, 13), read(src, 18), read(src, 23)] {
        [b'-', b'-', b'-', b'-'] => {}
        _ => return Err(Error::new()),
    }

    let mut flag = 0;
    let positions: [usize; 8] = [0, 4, 9, 14, 19, 24, 28, 32];
    for (j, i) in positions.iter().copied().enumerate() {
        let h1 = unhex(read(src, i));
        let h2 = unhex(read(src, i + 1));
        let h3 = unhex(read(src, i + 2));
        let h4 = unhex(read(src, i + 3));
        flag |= h1 | h2 | h3 | h4;
        write(dst, j * 2, shl4(h1) | h2);
        write(dst, j * 2 + 1, shl4(h3) | h4);
    }
    ensure!(flag != 0xff);
    Ok(())
}

#[inline(always)]
const fn char_lut(case: AsciiCase) -> &'static [u8; 16] {
    const LOWER_TABLE: &[u8; 16] = b"0123456789abcdef";
    const UPPER_TABLE: &[u8; 16] = b"0123456789ABCDEF";

    match case {
        AsciiCase::Lower => LOWER_TABLE,
        AsciiCase::Upper => UPPER_TABLE,
    }
}

#[inline]
pub unsafe fn format_simple(src: *const u8, dst: *mut u8, case: AsciiCase) {
    let lut = char_lut(case).as_ptr();
    for i in 0..16 {
        let x = read(src, i);
        let hi = read(lut, (x >> 4) as usize);
        let lo = read(lut, (x & 0x0f) as usize);
        write(dst, i * 2, hi);
        write(dst, i * 2 + 1, lo);
    }
}

#[inline]
pub unsafe fn format_hyphenated(src: *const u8, dst: *mut u8, case: AsciiCase) {
    let lut = char_lut(case).as_ptr();
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
