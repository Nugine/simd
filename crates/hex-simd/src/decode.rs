use crate::Error;

use vsimd::hex::unhex;
use vsimd::tools::read;
use vsimd::vector::V64;
use vsimd::SIMD256;

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
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
    {
        if cfg!(target_feature = "sse2") {
            return self::sse2::decode(src, len, dst);
        }
    }

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

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
mod sse2 {
    use super::*;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn decode(mut src: *const u8, mut len: usize, mut dst: *mut u8) -> Result<(), Error> {
        let end = src.add(len / 16 * 16);
        while src < end {
            let x = _mm_loadu_si128(src.cast());

            let (nibbles, flag) = decode_nibbles(x);
            ensure!(_mm_movemask_epi8(flag) == 0);

            let ans = merge_bits(nibbles);
            dst.cast::<u64>().write_unaligned(ans);

            src = src.add(16);
            dst = dst.add(8);
        }
        len %= 16;

        if len > 0 {
            decode_short(src, len, dst)?;
        }

        Ok(())
    }

    #[inline(always)]
    unsafe fn _mm_set1_epu8(x: u8) -> __m128i {
        _mm_set1_epi8(x as i8)
    }

    #[inline(always)]
    unsafe fn decode_nibbles(x: __m128i) -> (__m128i, __m128i) {
        // http://0x80.pl/notesen/2022-01-17-validating-hex-parse.html
        // Algorithm 3

        let t1 = _mm_add_epi8(x, _mm_set1_epu8(0xff - b'9'));
        let t2 = _mm_subs_epu8(t1, _mm_set1_epi8(6));
        let t3 = _mm_sub_epi8(t2, _mm_set1_epu8(0xf0));
        let t4 = _mm_and_si128(x, _mm_set1_epu8(0xdf));
        let t5 = _mm_sub_epi8(t4, _mm_set1_epi8(0x41));
        let t6 = _mm_adds_epu8(t5, _mm_set1_epi8(10));
        let t7 = _mm_min_epu8(t3, t6);
        let t8 = _mm_adds_epu8(t7, _mm_set1_epi8(127 - 15));
        (t7, t8)
    }

    #[inline(always)]
    unsafe fn merge_bits(x: __m128i) -> u64 {
        let lo = _mm_srli_epi16::<8>(x);
        let hi = _mm_slli_epi16::<4>(x);
        let t1 = _mm_or_si128(lo, hi);
        let t2 = _mm_and_si128(t1, _mm_set1_epi16(0xff));
        let t3 = _mm_packus_epi16(t2, _mm_setzero_si128());
        let [ans, _]: [u64; 2] = core::mem::transmute(t3);
        ans
    }
}
