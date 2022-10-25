use crate::Error;

use vsimd::hex::unhex;
use vsimd::isa::{InstructionSet, AVX2, SSE2};
use vsimd::tools::{is_same_type, read};
use vsimd::vector::V64;
use vsimd::{is_subtype, SIMD128, SIMD256};

#[inline(always)]
fn shl4(x: u8) -> u8 {
    x.wrapping_shl(4)
}

#[inline(always)]
unsafe fn decode_bits(src: *const u8, dst: *mut u8) -> u8 {
    let y1 = unhex(read(src, 0));
    let y2 = unhex(read(src, 1));
    let z = shl4(y1) | y2;
    dst.write(z);
    y1 | y2
}

#[inline(always)]
unsafe fn decode_short(mut src: *const u8, len: usize, mut dst: *mut u8) -> Result<(), Error> {
    let end = src.add(len);
    let mut flag = 0;
    while src < end {
        flag |= decode_bits(src, dst);
        src = src.add(2);
        dst = dst.add(1);
    }
    ensure!(flag != 0xff);
    Ok(())
}

#[inline(always)]
unsafe fn decode_long(mut src: *const u8, len: usize, mut dst: *mut u8) -> Result<(), Error> {
    let end = src.add(len / 16 * 16);
    while src < end {
        let mut flag = 0;
        let mut i = 0;
        while i < 8 {
            flag |= decode_bits(src, dst);
            src = src.add(2);
            dst = dst.add(1);
            i += 1;
        }
        ensure!(flag != 0xff);
    }
    decode_short(src, len % 16, dst)
}

#[inline(always)]
pub unsafe fn decode_fallback(src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error> {
    decode_long(src, len, dst)
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
pub unsafe fn decode_simd<S: SIMD256>(s: S, src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error> {
    if is_same_type::<S, SSE2>() {
        return decode_simd_sse2(SSE2::new(), src, len, dst);
    }
    if is_subtype!(S, AVX2) {
        return decode_simd_v256(s, src, len, dst);
    }
    decode_simd_v128(s, src, len, dst)
}

#[inline(always)]
pub unsafe fn decode_simd_v256<S: SIMD256>(
    s: S,
    mut src: *const u8,
    mut len: usize,
    mut dst: *mut u8,
) -> Result<(), Error> {
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

    decode_short(src, len, dst)
}

#[inline(always)]
pub unsafe fn decode_simd_v128<S: SIMD256>(
    s: S,
    mut src: *const u8,
    mut len: usize,
    mut dst: *mut u8,
) -> Result<(), Error> {
    let end = src.add(len / 32 * 32);
    while src < end {
        decode32(s, src, dst)?;
        src = src.add(32);
        dst = dst.add(16);
    }
    len %= 32;

    if len == 0 {
        return Ok(());
    }
    if len >= 16 {
        decode16(s, src, dst)?;
        src = src.add(16);
        dst = dst.add(8);
        len -= 16;
    }
    decode_short(src, len, dst)
}

#[inline(always)]
pub unsafe fn decode_simd_sse2(s: SSE2, mut src: *const u8, mut len: usize, mut dst: *mut u8) -> Result<(), Error> {
    let end = src.add(len / 16 * 16);
    while src < end {
        let x = s.v128_load_unaligned(src);

        let (nibbles, flag) = vsimd::hex::sse2::decode_nibbles(s, x);
        ensure!(s.u8x16_bitmask(flag) == 0);

        let ans = vsimd::hex::sse2::merge_bits(s, nibbles);
        dst.cast::<V64>().write_unaligned(ans);

        src = src.add(16);
        dst = dst.add(8);
    }
    len %= 16;

    if len == 0 {
        return Ok(());
    }

    decode_short(src, len, dst)
}
