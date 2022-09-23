use crate::error::Error;

use vsimd::hex::unhex;
use vsimd::tools::{read, write};
use vsimd::vector::V64;
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

unsafe fn decode16<S: SIMD256>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let x = s.v128_load_unaligned(src);
    let y = vsimd::hex::decode_ascii16(s, x).map_err(|()| Error::new())?;
    dst.cast::<V64>().write_unaligned(y);
    Ok(())
}

unsafe fn decode32<S: SIMD256>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let x = s.v256_load_unaligned(src);
    let y = vsimd::hex::decode_ascii32(s, x).map_err(|()| Error::new())?;
    s.v128_store_unaligned(dst, y);
    Ok(())
}

#[inline]
pub unsafe fn decode_simd<S: SIMD256>(s: S, mut src: *const u8, mut len: usize, mut dst: *mut u8) -> Result<(), Error> {
    if len == 16 {
        return decode16(s, src, dst);
    }

    if len == 32 {
        return decode32(s, src, dst);
    }

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

    if len == 0 {
        return Ok(());
    }

    if len >= 32 {
        decode32(s, src, dst)?;
        src = src.add(32);
        dst = dst.add(16);
        len -= 32;
    }

    if len >= 16 {
        decode16(s, src, dst)?;
        src = src.add(16);
        dst = dst.add(8);
        len -= 16;
    }

    decode_fallback(src, len, dst)
}
