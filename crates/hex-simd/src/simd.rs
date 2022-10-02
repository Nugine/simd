use crate::fallback;
use crate::Error;

use vsimd::ascii::AsciiCase;
use vsimd::tools::slice;
use vsimd::tools::slice_parts;
use vsimd::vector::V64;
use vsimd::SIMD256;

#[inline(always)]
pub fn check<S: SIMD256>(s: S, data: &[u8]) -> Result<(), Error> {
    unsafe {
        let (mut src, mut len) = (data.as_ptr(), data.len());

        if len == 16 {
            let x = s.v128_load_unaligned(src);
            ensure!(vsimd::hex::check_ascii_xn(s, x));
            return Ok(());
        }

        if len == 32 {
            let x = s.v256_load_unaligned(src);
            ensure!(vsimd::hex::check_ascii_xn(s, x));
            return Ok(());
        }

        let end = src.add(len / 32 * 32);
        while src < end {
            let x = s.v256_load_unaligned(src);
            ensure!(vsimd::hex::check_ascii_xn(s, x));
            src = src.add(32);
        }
        len %= 32;

        if len == 0 {
            return Ok(());
        }

        if len >= 16 {
            let x = s.v128_load_unaligned(src);
            ensure!(vsimd::hex::check_ascii_xn(s, x));
            len -= 16;
            src = src.add(16);
        }

        fallback::check(slice(src, len))
    }
}

#[inline(always)]
pub unsafe fn encode<S: SIMD256>(s: S, src: &[u8], mut dst: *mut u8, case: AsciiCase) {
    let lut = match case {
        AsciiCase::Lower => vsimd::hex::ENCODE_LOWER_LUT,
        AsciiCase::Upper => vsimd::hex::ENCODE_UPPER_LUT,
    };

    let (mut src, mut len) = slice_parts(src);

    if len == 16 {
        let x = s.v128_load_unaligned(src);
        let y = vsimd::hex::encode_bytes16(s, x, lut);
        s.v256_store_unaligned(dst, y);
        return;
    }

    if len == 32 {
        let x = s.v256_load_unaligned(src);
        let (y1, y2) = vsimd::hex::encode_bytes32(s, x, lut);
        s.v256_store_unaligned(dst, y1);
        s.v256_store_unaligned(dst.add(32), y2);
        return;
    }

    let end = src.add(len / 32 * 32);
    while src < end {
        let x = s.v256_load_unaligned(src);
        let (y1, y2) = vsimd::hex::encode_bytes32(s, x, lut);

        s.v256_store_unaligned(dst, y1);
        dst = dst.add(32);

        s.v256_store_unaligned(dst, y2);
        dst = dst.add(32);

        src = src.add(32);
    }
    len %= 32;

    if len == 0 {
        return;
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
        let table = match case {
            AsciiCase::Lower => fallback::FULL_LOWER_TABLE,
            AsciiCase::Upper => fallback::FULL_UPPER_TABLE,
        };
        fallback::encode_short(src, len, dst, table.as_ptr());
    }
}

#[inline(always)]
unsafe fn decode16<S: SIMD256>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let x = s.v128_load_unaligned(src);
    let y = try_!(vsimd::hex::decode_ascii16(s, x));
    dst.cast::<V64>().write_unaligned(y);
    Ok(())
}

#[inline(always)]
unsafe fn decode32<S: SIMD256>(s: S, src: *const u8, dst: *mut u8) -> Result<(), Error> {
    let x = s.v256_load_unaligned(src);
    let y = try_!(vsimd::hex::decode_ascii32(s, x));
    s.v128_store_unaligned(dst, y);
    Ok(())
}

#[inline(always)]
pub unsafe fn decode<S: SIMD256>(s: S, mut src: *const u8, mut len: usize, mut dst: *mut u8) -> Result<(), Error> {
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
        let y = try_!(vsimd::hex::decode_ascii32x2(s, x));
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

    fallback::decode_short(src, len, dst)
}
