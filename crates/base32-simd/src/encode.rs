use crate::Kind;

use vsimd::base32::{BASE32HEX_CHARSET, BASE32_CHARSET};
use vsimd::base32::{BASE32HEX_ENCODING_LUT, BASE32_ENCODING_LUT};
use vsimd::tools::{read, slice, slice_parts, write};
use vsimd::SIMD256;

pub const fn encoded_length_unchecked(len: usize, padding: bool) -> usize {
    let l = len / 5 * 8;
    if len % 5 == 0 {
        return l;
    }
    if padding {
        return l + 8;
    }
    const EXTRA: [u8; 5] = [0, 2, 4, 5, 7];
    l + EXTRA[len % 5] as usize
}

#[inline(always)]
pub unsafe fn encode_bits<const N: usize>(dst: *mut u8, charset: *const u8, x: u64) {
    debug_assert!(matches!(N, 2 | 4 | 5 | 7 | 8));

    {
        let shift = (N - 1) * 5;
        write(dst, 0, read(charset, (x >> shift) as usize));
    }
    let mut i = 1;
    while i < N {
        let shift = (N - 1 - i) * 5;
        write(dst, i, read(charset, ((x >> shift) & 0x1f) as usize));
        i += 1;
    }
}

#[inline(always)]
pub unsafe fn read_be_bytes<const N: usize>(src: *const u8) -> u64 {
    debug_assert!(matches!(N, 1 | 2 | 3 | 4 | 5));

    #[cfg(not(target_arch = "wasm32"))]
    {
        if N == 3 {
            let x1: u8 = read(src, 0);
            let x2: u16 = src.add(1).cast::<u16>().read_unaligned().to_be();
            return ((x1 as u64) << 16) | (x2 as u64);
        }
        if N == 5 {
            let x1: u8 = read(src, 0);
            let x2: u32 = src.add(1).cast::<u32>().read_unaligned().to_be();
            return ((x1 as u64) << 32) | (x2 as u64);
        }
    }

    let mut ans = 0;
    let mut i = 0;
    while i < N {
        let shift = (N - 1 - i) * 8;
        ans |= (read(src, i) as u64) << shift;
        i += 1;
    }
    ans
}

#[inline(always)]
unsafe fn encode_extra(src: *const u8, extra: usize, dst: *mut u8, charset: *const u8, padding: bool) {
    match extra {
        0 => {}
        1 => {
            let u10 = read_be_bytes::<1>(src) << 2;
            encode_bits::<2>(dst, charset, u10);
            if padding {
                let mut i = 2;
                while i < 8 {
                    write(dst, i, b'=');
                    i += 1;
                }
            }
        }
        2 => {
            let u20 = read_be_bytes::<2>(src) << 4;
            encode_bits::<4>(dst, charset, u20);
            if padding {
                let mut i = 4;
                while i < 8 {
                    write(dst, i, b'=');
                    i += 1;
                }
            }
        }
        3 => {
            let u25 = read_be_bytes::<3>(src) << 1;
            encode_bits::<5>(dst, charset, u25);
            if padding {
                let mut i = 5;
                while i < 8 {
                    write(dst, i, b'=');
                    i += 1;
                }
            }
        }
        4 => {
            let u35 = read_be_bytes::<4>(src) << 3;
            encode_bits::<7>(dst, charset, u35);
            if padding {
                write(dst, 7, b'=');
            }
        }
        _ => core::hint::unreachable_unchecked(),
    }
}

#[inline(always)]
pub unsafe fn encode_fallback(src: &[u8], mut dst: *mut u8, kind: Kind, padding: bool) {
    let charset: *const u8 = match kind {
        Kind::Base32 => BASE32_CHARSET.as_ptr(),
        Kind::Base32Hex => BASE32HEX_CHARSET.as_ptr(),
    };

    let (mut src, mut len) = slice_parts(src);

    let end = src.add(len / 5 * 5);
    while src < end {
        let u40 = read_be_bytes::<5>(src);
        encode_bits::<8>(dst, charset, u40);
        src = src.add(5);
        dst = dst.add(8);
    }
    len %= 5;

    encode_extra(src, len, dst, charset, padding);
}

#[inline(always)]
pub unsafe fn encode_simd<S: SIMD256>(s: S, src: &[u8], mut dst: *mut u8, kind: Kind, padding: bool) {
    let (charset, encoding_lut) = match kind {
        Kind::Base32 => (BASE32_CHARSET.as_ptr(), BASE32_ENCODING_LUT),
        Kind::Base32Hex => (BASE32HEX_CHARSET.as_ptr(), BASE32HEX_ENCODING_LUT),
    };

    let (mut src, mut len) = slice_parts(src);

    if len >= (10 + 20 + 6) {
        {
            let u40 = read_be_bytes::<5>(src);
            encode_bits::<8>(dst, charset, u40);
            src = src.add(5);
            dst = dst.add(8);

            let u40 = read_be_bytes::<5>(src);
            encode_bits::<8>(dst, charset, u40);
            src = src.add(5);
            dst = dst.add(8);

            len -= 10;
        }

        while len >= (20 + 6) {
            let x = s.v256_load_unaligned(src.sub(6));
            let y = vsimd::base32::encode_bytes20(s, x, encoding_lut);
            s.v256_store_unaligned(dst, y);
            src = src.add(20);
            dst = dst.add(32);
            len -= 20;
        }
    }

    encode_fallback(slice(src, len), dst, kind, padding);
}
