use vsimd::ascii::AsciiCase;
use vsimd::isa::{InstructionSet, AVX2, SSE2};
use vsimd::tools::{is_same_type, read, write};
use vsimd::{is_subtype, SIMD128, SIMD256};

#[inline(always)]
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
    encode_long(src, len, dst, case);
}

#[inline(always)]
pub unsafe fn encode_simd<S: SIMD256>(s: S, src: *const u8, len: usize, dst: *mut u8, case: AsciiCase) {
    if is_same_type::<S, SSE2>() {
        return encode_simd_sse2(SSE2::new(), src, len, dst, case);
    }
    if is_subtype!(S, AVX2) {
        return encode_simd_v256(s, src, len, dst, case);
    }
    encode_simd_v128(s, src, len, dst, case);
}

#[inline(always)]
pub unsafe fn encode_simd_v256<S: SIMD256>(
    s: S,
    mut src: *const u8,
    mut len: usize,
    mut dst: *mut u8,
    case: AsciiCase,
) {
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

#[inline(always)]
pub unsafe fn encode_simd_v128<S: SIMD256>(
    s: S,
    mut src: *const u8,
    mut len: usize,
    mut dst: *mut u8,
    case: AsciiCase,
) {
    let lut = match case {
        AsciiCase::Lower => vsimd::hex::ENCODE_LOWER_LUT,
        AsciiCase::Upper => vsimd::hex::ENCODE_UPPER_LUT,
    };

    let end = src.add(len / 16 * 16);
    while src < end {
        let x = s.v128_load_unaligned(src);
        let y = vsimd::hex::encode_bytes16(s, x, lut);
        s.v256_store_unaligned(dst, y);
        dst = dst.add(32);
        src = src.add(16);
    }
    len %= 16;

    if len == 0 {
        return;
    }

    let charset = charset(case).as_ptr();
    encode_short(src, len, dst, charset);
}

#[inline(always)]
pub unsafe fn encode_simd_sse2(s: SSE2, mut src: *const u8, mut len: usize, mut dst: *mut u8, case: AsciiCase) {
    let offset = match case {
        AsciiCase::Lower => vsimd::hex::sse2::LOWER_OFFSET,
        AsciiCase::Upper => vsimd::hex::sse2::UPPER_OFFSET,
    };

    let end = src.add(len / 16 * 16);
    while src < end {
        let x = s.v128_load_unaligned(src);
        src = src.add(16);

        let (y1, y2) = vsimd::hex::sse2::encode16(s, x, offset);

        s.v128_store_unaligned(dst, y1);
        dst = dst.add(16);

        s.v128_store_unaligned(dst, y2);
        dst = dst.add(16);
    }
    len %= 16;

    if len == 0 {
        return;
    }

    let charset = charset(case).as_ptr();
    encode_short(src, len, dst, charset);
}
