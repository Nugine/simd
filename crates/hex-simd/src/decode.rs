use crate::error::Error;

use vsimd::hex::unhex;
use vsimd::tools::{read, write};
use vsimd::SIMD256;

#[inline(always)]
fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

#[inline]
pub unsafe fn decode_fallback(src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error> {
    for i in 0..len / 2 {
        let y1 = unhex(read(src, i * 2));
        let y2 = unhex(read(src, i * 2 + 1));
        ensure!((y1 | y2) != 0xff);
        let z = shl4(y1) | y2;
        write(dst, i, z);
    }
    Ok(())
}

#[inline]
pub unsafe fn decode_simd<S: SIMD256>(s: S, mut src: *const u8, mut len: usize, mut dst: *mut u8) -> Result<(), Error> {
    let end = src.add(len / 64 * 64);
    while src < end {
        let x0 = s.v256_load_unaligned(src);
        src = src.add(32);

        let x1 = s.v256_load_unaligned(src);
        src = src.add(32);

        let x = (x0, x1);
        let y = vsimd::hex::decode_ascii32x2(s, x).map_err(|()| Error::new())?;
        s.v256_store_unaligned(dst, y);
        dst = dst.add(32);
    }
    len %= 64;

    if len >= 32 {
        let x = s.v256_load_unaligned(src);
        src = src.add(32);

        let y = vsimd::hex::decode_ascii32(s, x).map_err(|()| Error::new())?;
        s.v128_store_unaligned(dst, y);
        dst = dst.add(16);

        len -= 32;
    }

    if len > 0 {
        decode_fallback(src, len, dst)?;
    }
    Ok(())
}
