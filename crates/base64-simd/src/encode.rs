use crate::{Config, Kind};

use vsimd::base64::{STANDARD_CHARSET, URL_SAFE_CHARSET};
use vsimd::tools::{read, slice, slice_parts, write};
use vsimd::SIMD256;

#[inline(always)]
pub(crate) const fn encoded_length_unchecked(len: usize, config: Config) -> usize {
    let extra = len % 3;
    if extra == 0 {
        len / 3 * 4
    } else if config.extra.padding() {
        len / 3 * 4 + 4
    } else {
        len / 3 * 4 + extra + 1
    }
}

#[inline(always)]
unsafe fn encode_bits24(src: *const u8, dst: *mut u8, charset: *const u8) {
    let x = u32::from_be_bytes([0, read(src, 0), read(src, 1), read(src, 2)]);
    let mut i = 0;
    while i < 4 {
        let bits = (x >> (18 - i * 6)) & 0x3f;
        let y = read(charset, bits as usize);
        write(dst, i, y);
        i += 1;
    }
}

#[inline(always)]
unsafe fn encode_bits48(src: *const u8, dst: *mut u8, charset: *const u8) {
    let x = u64::from_be_bytes(src.cast::<[u8; 8]>().read());
    let mut i = 0;
    while i < 8 {
        let bits = (x >> (58 - i * 6)) & 0x3f;
        let y = read(charset, bits as usize);
        write(dst, i, y);
        i += 1;
    }
}

#[inline(always)]
unsafe fn encode_extra(extra: usize, src: *const u8, dst: *mut u8, charset: *const u8, padding: bool) {
    match extra {
        0 => {}
        1 => {
            let x = read(src, 0);
            let y1 = read(charset, (x >> 2) as usize);
            let y2 = read(charset, ((x << 6) >> 2) as usize);
            write(dst, 0, y1);
            write(dst, 1, y2);
            if padding {
                write(dst, 2, b'=');
                write(dst, 3, b'=');
            }
        }
        2 => {
            let x1 = read(src, 0);
            let x2 = read(src, 1);
            let y1 = read(charset, (x1 >> 2) as usize);
            let y2 = read(charset, (((x1 << 6) >> 2) | (x2 >> 4)) as usize);
            let y3 = read(charset, ((x2 << 4) >> 2) as usize);
            write(dst, 0, y1);
            write(dst, 1, y2);
            write(dst, 2, y3);
            if padding {
                write(dst, 3, b'=');
            }
        }
        _ => core::hint::unreachable_unchecked(),
    }
}

pub(crate) unsafe fn encode_fallback(src: &[u8], mut dst: *mut u8, config: Config) {
    let kind = config.kind;
    let padding = config.extra.padding();

    let charset = match kind {
        Kind::Standard => STANDARD_CHARSET.as_ptr(),
        Kind::UrlSafe => URL_SAFE_CHARSET.as_ptr(),
    };

    let (mut src, mut len) = slice_parts(src);

    const L: usize = 4;
    while len >= L * 6 + 2 {
        let mut i = 0;
        while i < L {
            encode_bits48(src, dst, charset);
            src = src.add(6);
            dst = dst.add(8);
            i += 1;
        }
        len -= L * 6;
    }

    while len >= 6 + 2 {
        encode_bits48(src, dst, charset);
        src = src.add(6);
        dst = dst.add(8);
        len -= 6;
    }

    let end = src.add(len / 3 * 3);
    while src < end {
        encode_bits24(src, dst, charset);
        src = src.add(3);
        dst = dst.add(4);
    }
    len %= 3;

    encode_extra(len, src, dst, charset, padding);
}

pub(crate) unsafe fn encode_simd<S: SIMD256>(s: S, src: &[u8], mut dst: *mut u8, config: Config) {
    let kind = config.kind;

    let (mut src, mut len) = slice_parts(src);

    if len >= (6 + 24 + 4) {
        let (charset, shift_lut) = match kind {
            Kind::Standard => (STANDARD_CHARSET.as_ptr(), vsimd::base64::STANDARD_ENCODING_SHIFT_X2),
            Kind::UrlSafe => (URL_SAFE_CHARSET.as_ptr(), vsimd::base64::URL_SAFE_ENCODING_SHIFT_X2),
        };

        for _ in 0..2 {
            encode_bits24(src, dst, charset);
            src = src.add(3);
            dst = dst.add(4);
            len -= 3;
        }

        while len >= (24 + 4) {
            let x = s.v256_load_unaligned(src.sub(4));
            let y = vsimd::base64::encode_bytes24(s, x, shift_lut);
            s.v256_store_unaligned(dst, y);
            src = src.add(24);
            dst = dst.add(32);
            len -= 24;
        }
    }

    if len >= 12 + 4 {
        let shift_lut = match kind {
            Kind::Standard => vsimd::base64::STANDARD_ENCODING_SHIFT,
            Kind::UrlSafe => vsimd::base64::URL_SAFE_ENCODING_SHIFT,
        };

        let x = s.v128_load_unaligned(src);
        let y = vsimd::base64::encode_bytes12(s, x, shift_lut);
        s.v128_store_unaligned(dst, y);
        src = src.add(12);
        dst = dst.add(16);
        len -= 12;
    }

    encode_fallback(slice(src, len), dst, config);
}
