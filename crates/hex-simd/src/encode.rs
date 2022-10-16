use vsimd::ascii::AsciiCase;
use vsimd::tools::{read, slice_parts, write};
use vsimd::SIMD256;

fn charset(case: AsciiCase) -> &'static [u8; 16] {
    match case {
        AsciiCase::Lower => vsimd::hex::LOWER_CHARSET,
        AsciiCase::Upper => vsimd::hex::UPPER_CHARSET,
    }
}

#[inline(always)]
unsafe fn encode_bits(src: *const u8, dst: *mut u8, charset: *const u8) {
    let x = src.read();
    let hi = read(charset, (x >> 4) as usize);
    let lo = read(charset, (x & 0x0f) as usize);
    write(dst, 0, hi);
    write(dst, 1, lo);
}

#[inline(always)]
unsafe fn encode_short(mut src: *const u8, len: usize, mut dst: *mut u8, charset: *const u8) {
    let end = src.add(len);
    while src < end {
        encode_bits(src, dst, charset);
        src = src.add(1);
        dst = dst.add(2);
    }
}

unsafe fn encode_long(src: &[u8], mut dst: *mut u8, case: AsciiCase) {
    let charset = charset(case).as_ptr();

    let (mut src, len) = (src.as_ptr(), src.len());

    let end = src.add(len / 8 * 8);
    while src < end {
        let mut i = 0;
        while i < 8 {
            encode_bits(src, dst, charset);
            src = src.add(1);
            dst = dst.add(2);
            i += 1;
        }
    }
    encode_short(src, len % 8, dst, charset);
}

#[inline(always)]
pub unsafe fn encode_fallback(src: &[u8], dst: *mut u8, case: AsciiCase) {
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
    {
        if cfg!(target_feature = "sse2") {
            self::sse2::encode(src, dst, case);
            return;
        }
    }

    encode_long(src, dst, case);
}

#[inline(always)]
pub unsafe fn encode_simd<S: SIMD256>(s: S, src: &[u8], mut dst: *mut u8, case: AsciiCase) {
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
        let charset = charset(case).as_ptr();
        encode_short(src, len, dst, charset);
    }
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
    pub unsafe fn encode(src: &[u8], mut dst: *mut u8, case: AsciiCase) {
        let (mut src, mut len) = slice_parts(src);

        let offset = match case {
            AsciiCase::Lower => _mm_set1_epi8(0x27),
            AsciiCase::Upper => _mm_set1_epi8(0x07),
        };

        let encode_bytes16 = |x: __m128i, o: __m128i| {
            let t = _mm_add_epi8(x, _mm_set1_epi8(0x30));
            let cmp = _mm_cmplt_epi8(_mm_set1_epi8(0x39), t);
            let offset = _mm_and_si128(cmp, o);
            _mm_add_epi8(t, offset)
        };

        let end = src.add(len / 16 * 16);
        while src < end {
            let x = _mm_loadu_si128(src.cast());
            src = src.add(16);

            let m = _mm_set1_epi8(0x0f);
            let hi = _mm_and_si128(_mm_srli_epi16::<4>(x), m);
            let lo = _mm_and_si128(x, m);

            let hi = encode_bytes16(hi, offset);
            let lo = encode_bytes16(lo, offset);

            let y1 = _mm_unpacklo_epi8(hi, lo);
            let y2 = _mm_unpackhi_epi8(hi, lo);

            _mm_storeu_si128(dst.cast(), y1);
            dst = dst.add(16);

            _mm_storeu_si128(dst.cast(), y2);
            dst = dst.add(16);
        }
        len %= 16;

        if len > 0 {
            let charset = charset(case).as_ptr();
            encode_short(src, len, dst, charset);
        }
    }
}
