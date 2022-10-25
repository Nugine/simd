use crate::{u16x4_to_u64, Kind};
use crate::{BASE32HEX_CHARSET, BASE32_CHARSET};

use vsimd::isa::{NEON, SSE41, SSSE3, WASM128};
use vsimd::tools::{read, write};
use vsimd::vector::V256;
use vsimd::{is_subtype, SIMD256};

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
pub(crate) unsafe fn encode_fallback(mut src: *const u8, mut len: usize, mut dst: *mut u8, kind: Kind, padding: bool) {
    let charset: *const u8 = match kind {
        Kind::Base32 => BASE32_CHARSET.as_ptr(),
        Kind::Base32Hex => BASE32HEX_CHARSET.as_ptr(),
    };

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
pub(crate) unsafe fn encode_simd<S: SIMD256>(
    s: S,
    mut src: *const u8,
    mut len: usize,
    mut dst: *mut u8,
    kind: Kind,
    padding: bool,
) {
    let (charset, encoding_lut) = match kind {
        Kind::Base32 => (BASE32_CHARSET.as_ptr(), BASE32_ENCODING_LUT),
        Kind::Base32Hex => (BASE32HEX_CHARSET.as_ptr(), BASE32HEX_ENCODING_LUT),
    };

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
            let y = encode_bytes20(s, x, encoding_lut);
            s.v256_store_unaligned(dst, y);
            src = src.add(20);
            dst = dst.add(32);
            len -= 20;
        }
    }

    encode_fallback(src, len, dst, kind, padding);
}

#[inline(always)]
fn split_bits<S: SIMD256>(s: S, x: V256) -> V256 {
    const SPLIT_SHUFFLE: V256 = V256::from_bytes([
        0x07, 0x06, 0x08, 0x07, 0x09, 0x08, 0x0A, 0x09, //
        0x0C, 0x0B, 0x0D, 0x0C, 0x0E, 0x0D, 0x0F, 0x0E, //
        0x01, 0x00, 0x02, 0x01, 0x03, 0x02, 0x04, 0x03, //
        0x06, 0x05, 0x07, 0x06, 0x08, 0x07, 0x09, 0x08, //
    ]);

    if is_subtype!(S, SSSE3) {
        const SPLIT_M1: u64 = u16x4_to_u64([1 << 5, 1 << 7, 1 << 9, 1 << 11]);
        const SPLIT_M2: u64 = u16x4_to_u64([1 << 2, 1 << 4, 1 << 6, 1 << 8]);

        let x1 = s.u8x16x2_swizzle(x, SPLIT_SHUFFLE);
        let x2 = s.u16x16_mul_hi(x1, s.u64x4_splat(SPLIT_M1));
        let x3 = s.i16x16_mul_lo(x1, s.u64x4_splat(SPLIT_M2));
        let x4 = s.v256_and(x2, s.u16x16_splat(u16::from_le_bytes([0x1f, 0x00])));
        let x5 = s.v256_and(x3, s.u16x16_splat(u16::from_le_bytes([0x00, 0x1f])));
        return s.v256_or(x4, x5);
    }

    if is_subtype!(S, NEON | WASM128) {
        const SPLIT_M1: u64 = u16x4_to_u64([1 << 1, 1 << 3, 1 << 5, 1 << 7]);
        const SPLIT_M2: u64 = u16x4_to_u64([1 << 2, 1 << 4, 1 << 6, 1 << 8]);
        const SPLIT_M3: u16 = u16::from_le_bytes([0x00, 0x1f]);

        let x1 = s.u8x16x2_swizzle(x, SPLIT_SHUFFLE);
        let x2 = s.u16x16_shr::<4>(x1);
        let x3 = s.i16x16_mul_lo(x2, s.u64x4_splat(SPLIT_M1));
        let x4 = s.i16x16_mul_lo(x1, s.u64x4_splat(SPLIT_M2));
        let m3 = s.u16x16_splat(SPLIT_M3);
        let x5 = s.v256_and(x3, m3);
        let x6 = s.v256_and(x4, m3);
        let x7 = s.u16x16_shr::<8>(x5);
        return s.v256_or(x6, x7);
    }

    unreachable!()
}

#[derive(Debug, Clone, Copy)]
struct EncodingLutX2 {
    low: V256,
    high: V256,
    full: V256,
}

impl EncodingLutX2 {
    const fn new(charset: &[u8; 32]) -> Self {
        let full = V256::from_bytes(*charset);
        let charset: &[[u8; 16]; 2] = unsafe { core::mem::transmute(charset) };
        let low = V256::double_bytes(charset[0]);
        let high = V256::double_bytes(charset[1]);
        Self { low, high, full }
    }
}

const BASE32_ENCODING_LUT: EncodingLutX2 = EncodingLutX2::new(BASE32_CHARSET);
const BASE32HEX_ENCODING_LUT: EncodingLutX2 = EncodingLutX2::new(BASE32HEX_CHARSET);

#[inline(always)]
fn encode_values<S: SIMD256>(s: S, x: V256, lut: EncodingLutX2) -> V256 {
    if is_subtype!(S, SSE41) {
        let x1 = s.u8x16x2_swizzle(lut.low, x);
        let x2 = s.u8x16x2_swizzle(lut.high, x);
        let x3 = s.u8x32_lt(s.u8x32_splat(0x0f), x);
        return s.u8x32_blendv(x1, x2, x3);
    }
    if is_subtype!(S, NEON) && cfg!(target_arch = "aarch64") {
        return s.u8x32_swizzle(lut.full, x);
    }
    if is_subtype!(S, NEON | WASM128) {
        let m = s.u8x32_splat(0x0f);
        let x1 = s.v256_and(x, m);
        let x2 = s.u8x16x2_swizzle(lut.low, x1);
        let x3 = s.u8x16x2_swizzle(lut.high, x1);
        let x4 = s.u8x32_lt(m, x);
        return s.v256_bsl(x4, x3, x2);
    }
    unreachable!()
}

#[inline(always)]
fn encode_bytes20<S: SIMD256>(s: S, x: V256, lut: EncodingLutX2) -> V256 {
    // x: {????|??AA|AAAB|BBBB|CCCC|CDDD|DD??|????}

    let values = split_bits(s, x);
    // values: {000xyyyy}x32

    encode_values(s, values, lut)
    // {{ascii}}x32
}
