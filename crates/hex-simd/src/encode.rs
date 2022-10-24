use vsimd::ascii::AsciiCase;
use vsimd::tools::{read, write};
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

unsafe fn encode_long(mut src: *const u8, len: usize, mut dst: *mut u8, case: AsciiCase) {
    let charset = charset(case).as_ptr();

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
pub unsafe fn encode_fallback(src: *const u8, len: usize, dst: *mut u8, case: AsciiCase) {
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
    {
        if cfg!(target_feature = "sse2") {
            self::sse2::encode(src, len, dst, case);
            return;
        }
    }

    encode_long(src, len, dst, case);
}

#[inline(always)]
pub unsafe fn encode_simd<S: SIMD256>(s: S, mut src: *const u8, mut len: usize, mut dst: *mut u8, case: AsciiCase) {
    let lut = match case {
        AsciiCase::Lower => vsimd::hex::ENCODE_LOWER_LUT,
        AsciiCase::Upper => vsimd::hex::ENCODE_UPPER_LUT,
    };

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

    use vsimd::hex::sse2::*;
    use vsimd::isa::{InstructionSet, SSE2};
    use vsimd::SIMD128;

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn encode(mut src: *const u8, mut len: usize, mut dst: *mut u8, case: AsciiCase) {
        let s = SSE2::new();

        let offset = match case {
            AsciiCase::Lower => LOWER_OFFSET,
            AsciiCase::Upper => UPPER_OFFSET,
        };

        let end = src.add(len / 16 * 16);
        while src < end {
            let x = s.v128_load_unaligned(src);
            src = src.add(16);

            let (y1, y2) = encode16(s, x, offset);

            s.v128_store_unaligned(dst, y1);
            dst = dst.add(16);

            s.v128_store_unaligned(dst, y2);
            dst = dst.add(16);
        }
        len %= 16;

        if len > 0 {
            let charset = charset(case).as_ptr();
            encode_short(src, len, dst, charset);
        }
    }
}
