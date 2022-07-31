use crate::error::{Error, ERROR};
use crate::sa_hex::{self, unhex};

use simd_abstraction::isa::SIMD256;
use simd_abstraction::tools::{read, write};

#[inline(always)]
fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

#[inline]
pub unsafe fn decode_raw_fallback(src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error> {
    for i in 0..len / 2 {
        let y1 = unhex(read(src, i * 2));
        let y2 = unhex(read(src, i * 2 + 1));
        if y1 | y2 == 0xff {
            return Err(ERROR);
        }
        let z = shl4(y1) | y2;
        write(dst, i, z);
    }
    Ok(())
}

#[inline]
pub unsafe fn decode_raw_simd<S: SIMD256>(s: S, mut src: *const u8, len: usize, mut dst: *mut u8) -> Result<(), Error> {
    let end = src.add(len / 32 * 32);
    while src < end {
        let x = s.v256_load_unaligned(src);
        let y = sa_hex::decode_u8x32(s, x).map_err(|()| ERROR)?;
        s.v128_store_unaligned(dst, y);
        src = src.add(32);
        dst = dst.add(16);
    }
    decode_raw_fallback(src, len % 32, dst)?;
    Ok(())
}
