use crate::error::Error;
use crate::spec::*;

use vsimd::hex::unhex;
use vsimd::tools::{read, write};
use vsimd::vector::V256;
use vsimd::SIMD256;

#[inline(always)]
const fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

#[inline]
pub unsafe fn parse_simple_fallback(src: *const u8, dst: *mut u8) -> Result<(), Error> {
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
pub unsafe fn parse_simple_simd<S: SIMD256>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let x = s.v256_load_unaligned(src);
    let y = vsimd::hex::decode_ascii32(s, x).map_err(|()| Error::new())?;
    s.v128_store_unaligned(dst, y);
    Ok(())
}

#[inline]
pub unsafe fn parse_hyphenated_fallback(src: *const u8, dst: *mut u8) -> Result<(), Error> {
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

#[inline]
pub unsafe fn parse_hyphenated_simd<S: SIMD256>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
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
