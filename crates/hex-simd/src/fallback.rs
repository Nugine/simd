use crate::Error;

use vsimd::ascii::AsciiCase;
use vsimd::hex::unhex;
use vsimd::tools::{read, write};

#[inline]
pub fn check(data: &[u8]) -> Result<(), Error> {
    let mut iter = data.chunks_exact(4);
    for chunk in &mut iter {
        let y1 = unhex(chunk[0]);
        let y2 = unhex(chunk[1]);
        let y3 = unhex(chunk[2]);
        let y4 = unhex(chunk[3]);
        ensure!((y1 | y2 | y3 | y4) != 0xff);
    }
    let flag = iter.remainder().iter().fold(0, |acc, &x| acc | unhex(x));
    ensure!(flag != 0xff);
    Ok(())
}

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
pub unsafe fn encode(src: &[u8], mut dst: *mut u8, case: AsciiCase) {
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
fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

#[inline]
pub unsafe fn decode(src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error> {
    for i in 0..len / 2 {
        let y1 = unhex(read(src, i * 2));
        let y2 = unhex(read(src, i * 2 + 1));
        ensure!((y1 | y2) != 0xff);
        let z = shl4(y1) | y2;
        write(dst, i, z);
    }
    Ok(())
}
