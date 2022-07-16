#![allow(missing_docs)]

use crate::sa_hex::unhex;
use crate::{AsciiCase, Error, OutBuf, ERROR};

use simd_abstraction::tools::{read, write};

use core::slice;

#[inline(always)]
fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

#[inline]
pub fn check(src: &[u8]) -> bool {
    #[inline(always)]
    unsafe fn check_unroll1(n: usize, src: *const u8) -> bool {
        let mut i = 0;
        let mut ans = 0;
        while i < n {
            ans |= unhex(read(src, i));
            i += 1;
        }
        ans != 0xff
    }
    #[inline(always)]
    unsafe fn check_unroll4(n: usize, src: *const u8) -> bool {
        let mut i = 0;
        while i < n {
            let y1 = unhex(read(src, i));
            let y2 = unhex(read(src, i + 1));
            let y3 = unhex(read(src, i + 2));
            let y4 = unhex(read(src, i + 3));
            if y1 | y2 | y3 | y4 == 0xff {
                return false;
            }
            i += 4;
        }
        true
    }

    let n = src.len();
    let src = src.as_ptr();
    unsafe {
        let n1 = n & 3;
        let n4 = n - n1;
        if n4 > 0 && !check_unroll4(n4, src) {
            return false;
        }
        check_unroll1(n1, src.add(n4))
    }
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

pub const FULL_LOWER_TABLE: &[u16; 256] = &full_table(LOWER_TABLE);
pub const FULL_UPPER_TABLE: &[u16; 256] = &full_table(UPPER_TABLE);

#[inline]
pub fn encode<'s, 'd>(
    src: &'s [u8],
    mut dst: OutBuf<'d>,
    case: AsciiCase,
) -> Result<&'d mut [u8], Error> {
    if dst.len() / 2 < src.len() {
        return Err(ERROR);
    }
    let table = match case {
        AsciiCase::Lower => FULL_LOWER_TABLE,
        AsciiCase::Upper => FULL_UPPER_TABLE,
    };
    unsafe {
        let dst = dst.as_mut_ptr();
        encode_unchecked(src, dst, table);
        Ok(slice::from_raw_parts_mut(dst, src.len() * 2))
    }
}

#[inline(always)]
pub(crate) unsafe fn encode_unchecked(src: &[u8], dst: *mut u8, table: &[u16; 256]) {
    let (n, src) = (src.len(), src.as_ptr());
    let table = table.as_ptr();
    let mut i = 0;
    while i < n {
        let x = read(src, i);
        let y = read(table, x as usize);
        dst.add(i * 2).cast::<u16>().write_unaligned(y);
        i += 1;
    }
}

#[inline]
pub fn decode<'s, 'd>(src: &'s [u8], mut dst: OutBuf<'d>) -> Result<&'d mut [u8], Error> {
    let n = src.len();
    let m = n / 2;
    if !(n % 2 == 0 && dst.len() >= m) {
        return Err(ERROR);
    }
    unsafe {
        let dst = dst.as_mut_ptr();
        decode_unchecked(m, src.as_ptr(), dst)?;
        Ok(slice::from_raw_parts_mut(dst, m))
    }
}

#[inline]
pub fn decode_inplace(buf: &mut [u8]) -> Result<&mut [u8], Error> {
    let n = buf.len();
    let m = n / 2;
    if n % 2 != 0 {
        return Err(ERROR);
    }
    unsafe {
        let dst: *mut u8 = buf.as_mut_ptr();
        let src: *const u8 = dst;
        decode_unchecked(m, src, dst)?;
        Ok(slice::from_raw_parts_mut(dst, m))
    }
}

/// `src` and `dst` may alias
#[inline(always)]
pub(crate) unsafe fn decode_unchecked(m: usize, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let mut i = 0;
    while i < m {
        let y1 = unhex(read(src, i * 2));
        let y2 = unhex(read(src, i * 2 + 1));
        if y1 | y2 == 0xff {
            return Err(ERROR);
        }
        let z = shl4(y1) | y2;
        write(dst, i, z);
        i += 1;
    }
    Ok(())
}

#[test]
fn test() {
    crate::tests::test(check, decode, encode, decode_inplace);
}
