use crate::alsw::{BASE32HEX_ALSW_CHECK_X2, BASE32HEX_ALSW_DECODE_X2};
use crate::alsw::{BASE32_ALSW_CHECK_X2, BASE32_ALSW_DECODE_X2};
use crate::{u16x4_to_u64, Error, Kind};
use crate::{BASE32HEX_CHARSET, BASE32_CHARSET};

use vsimd::alsw::AlswLut;
use vsimd::isa::{AVX2, NEON, SSE41, WASM128};
use vsimd::mask::u8x32_highbit_any;
use vsimd::tools::{read, write};
use vsimd::vector::V256;
use vsimd::{is_subtype, simd256_vop, SIMD256};

#[inline]
const fn decoding_table(charset: &[u8; 32]) -> [u8; 256] {
    let mut table = [0xff; 256];
    let mut i = 0;
    while i < 32 {
        table[charset[i] as usize] = i as u8;
        i += 1;
    }
    table
}

pub const BASE32_TABLE: &[u8; 256] = &decoding_table(BASE32_CHARSET);
pub const BASE32HEX_TABLE: &[u8; 256] = &decoding_table(BASE32HEX_CHARSET);

#[inline]
pub fn decoded_length(data: &[u8], padding: bool) -> Result<(usize, usize), Error> {
    if data.is_empty() {
        return Ok((0, 0));
    }

    let len = data.len();
    let n = if padding {
        ensure!(len % 8 == 0);
        let last = unsafe { data.get_unchecked(len - 6..) };
        let count = last.iter().copied().filter(|&x| x == b'=').count();
        len - count
    } else {
        data.len()
    };

    const EXTRA: [u8; 8] = [0, 0xff, 1, 0xff, 2, 3, 0xff, 4];
    let extra = EXTRA[n % 8];
    ensure!(extra != 0xff);
    let m = n / 8 * 5 + extra as usize;
    Ok((n, m))
}

#[inline(always)]
pub unsafe fn decode_bits<const N: usize>(src: *const u8, table: *const u8) -> (u64, u8) {
    debug_assert!(matches!(N, 2 | 4 | 5 | 7 | 8));
    let mut ans: u64 = 0;
    let mut flag = 0;
    let mut i = 0;
    while i < N {
        let bits = read(table, read(src, i) as usize);
        flag |= bits;
        ans = (ans << 5) | u64::from(bits);
        i += 1;
    }
    (ans, flag)
}

#[inline(always)]
unsafe fn write_be_bytes<const N: usize>(dst: *mut u8, x: u64) {
    debug_assert!(matches!(N, 1 | 2 | 3 | 4 | 5));

    #[cfg(not(target_arch = "wasm32"))]
    {
        if N == 3 {
            let x1 = (x >> 16) as u8;
            let x2 = (x as u16).to_be();
            dst.write(x1);
            dst.add(1).cast::<u16>().write_unaligned(x2);
            return;
        }
        if N == 5 {
            let x1 = (x >> 32) as u8;
            let x2 = (x as u32).to_be();
            dst.write(x1);
            dst.add(1).cast::<u32>().write_unaligned(x2);
            return;
        }
    }

    let mut i = 0;
    while i < N {
        let shift = (N - 1 - i) * 8;
        write(dst, i, (x >> shift) as u8);
        i += 1;
    }
}

#[inline(always)]
pub unsafe fn decode_extra<const WRITE: bool>(
    src: *const u8,
    extra: usize,
    dst: *mut u8,
    table: *const u8,
) -> Result<(), Error> {
    match extra {
        0 => {}
        2 => {
            let (u10, flag) = decode_bits::<2>(src, table);
            ensure!(flag != 0xff && u10 & 0b11 == 0);
            if WRITE {
                write_be_bytes::<1>(dst, u10 >> 2);
            }
        }
        4 => {
            let (u20, flag) = decode_bits::<4>(src, table);
            ensure!(flag != 0xff && u20 & 0b1111 == 0);
            if WRITE {
                write_be_bytes::<2>(dst, u20 >> 4);
            }
        }
        5 => {
            let (u25, flag) = decode_bits::<5>(src, table);
            ensure!(flag != 0xff && u25 & 0b1 == 0);
            if WRITE {
                write_be_bytes::<3>(dst, u25 >> 1);
            }
        }
        7 => {
            let (u35, flag) = decode_bits::<7>(src, table);
            ensure!(flag != 0xff && u35 & 0b111 == 0);
            if WRITE {
                write_be_bytes::<4>(dst, u35 >> 3);
            }
        }
        _ => core::hint::unreachable_unchecked(),
    }
    Ok(())
}

#[inline(always)]
pub(crate) unsafe fn decode_fallback(
    mut src: *const u8,
    mut n: usize,
    mut dst: *mut u8,
    kind: Kind,
) -> Result<(), Error> {
    let table = match kind {
        Kind::Base32 => BASE32_TABLE.as_ptr(),
        Kind::Base32Hex => BASE32HEX_TABLE.as_ptr(),
    };

    let end = src.add(n / 8 * 8);
    while src < end {
        let (u40, flag) = decode_bits::<8>(src, table);
        ensure!(flag != 0xff);
        write_be_bytes::<5>(dst, u40);
        src = src.add(8);
        dst = dst.add(5);
    }
    n %= 8;

    decode_extra::<true>(src, n, dst, table)
}

#[inline(always)]
pub(crate) unsafe fn decode_simd<S: SIMD256>(
    s: S,
    mut src: *const u8,
    mut n: usize,
    mut dst: *mut u8,
    kind: Kind,
) -> Result<(), Error> {
    let (check_lut, decode_lut) = match kind {
        Kind::Base32 => (BASE32_ALSW_CHECK_X2, BASE32_ALSW_DECODE_X2),
        Kind::Base32Hex => (BASE32HEX_ALSW_CHECK_X2, BASE32HEX_ALSW_DECODE_X2),
    };

    // n*5/8 >= 10+10+6
    while n >= 42 {
        let x = s.v256_load_unaligned(src);
        let y = try_!(decode_ascii32(s, x, check_lut, decode_lut));

        let (y1, y2) = y.to_v128x2();
        s.v128_store_unaligned(dst, y1);
        s.v128_store_unaligned(dst.add(10), y2);

        src = src.add(32);
        dst = dst.add(20);
        n -= 32;
    }

    decode_fallback(src, n, dst, kind)
}

#[inline(always)]
fn u32x8_blend_0x55<S: SIMD256>(s: S, a: V256, b: V256) -> V256 {
    if is_subtype!(S, AVX2) {
        return s.u32x8_blend::<0x55>(a, b);
    }
    if is_subtype!(S, SSE41) {
        return simd256_vop!(s, S::u16x8_blend::<0x33>, a, b);
    }
    unreachable!()
}

#[inline(always)]
fn merge_bits<S: SIMD256>(s: S, x: V256) -> V256 {
    if is_subtype!(S, SSE41) {
        const MERGE_M1: u32 = u32::from_le_bytes([1 << 7, 1 << 2, 1 << 5, 1 << 0]);
        const MERGE_S1: V256 = V256::double_bytes([
            0x01, 0x00, 0x02, 0x04, 0x06, //
            0x09, 0x08, 0x0A, 0x0C, 0x0E, //
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
        ]);
        const MERGE_S2: V256 = V256::double_bytes([
            0x80, 0x03, 0x05, 0x07, 0x80, //
            0x80, 0x0B, 0x0D, 0x0F, 0x80, //
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
        ]);

        let x1 = s.i16x16_maddubs(s.u32x8_splat(MERGE_M1), x);
        let x2 = s.u32x8_shl::<4>(x1);
        let x3 = u32x8_blend_0x55(s, x1, x2);
        let x4 = s.u8x16x2_swizzle(x3, MERGE_S1);
        let x5 = s.u8x16x2_swizzle(x3, MERGE_S2);
        return s.v256_or(x4, x5);
    }

    if is_subtype!(S, NEON | WASM128) {
        const MERGE_M1: u16 = u16::from_le_bytes([0x1f, 0x00]);
        const MERGE_M2: u64 = u16x4_to_u64([1 << 3, 1 << 1, 1 << 7, 1 << 5]);
        const MERGE_M3: u64 = u16x4_to_u64([1 << 6, 1 << 4, 1 << 2, 1 << 0]);

        const MERGE_S1: V256 = V256::double_bytes([
            0x00, 0x02, 0x05, 0x07, 0x06, 0x80, 0x80, 0x04, //
            0x08, 0x0A, 0x0D, 0x0F, 0x0E, 0x80, 0x80, 0x0C, //
        ]);
        const MERGE_S2: V256 = V256::double_bytes([
            0x01, 0x00, 0x02, 0x04, 0x06, 0x03, 0x80, 0x80, //
            0x09, 0x08, 0x0A, 0x0C, 0x0E, 0x0B, 0x80, 0x80, //
        ]);
        const MERGE_S3: V256 = V256::double_bytes([
            0x00, 0x01, 0x02, 0x03, 0x04, //
            0x08, 0x09, 0x0A, 0x0B, 0x0C, //
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
        ]);
        const MERGE_S4: V256 = V256::double_bytes([
            0x80, 0x05, 0x80, 0x07, 0x80, //
            0x80, 0x0D, 0x80, 0x0F, 0x80, //
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
        ]);

        let x1 = s.v256_and(x, s.u16x16_splat(MERGE_M1));
        let x2 = s.i16x16_mul_lo(x1, s.u64x4_splat(MERGE_M2));
        let x3 = s.u16x16_shr::<8>(x);
        let x4 = s.i16x16_mul_lo(x3, s.u64x4_splat(MERGE_M3));
        let x5 = s.u8x16x2_swizzle(x2, MERGE_S1);
        let x6 = s.u8x16x2_swizzle(x4, MERGE_S2);
        let x7 = s.v256_or(x5, x6);
        let x8 = s.u8x16x2_swizzle(x7, MERGE_S3);
        let x9 = s.u8x16x2_swizzle(x7, MERGE_S4);
        return s.v256_or(x8, x9);
    }

    unreachable!()
}

#[allow(clippy::result_unit_err)]
#[inline(always)]
fn decode_ascii32<S: SIMD256>(s: S, x: V256, check: AlswLut<V256>, decode: AlswLut<V256>) -> Result<V256, ()> {
    let (c1, c2) = vsimd::alsw::decode_ascii_xn(s, x, check, decode);

    let y = merge_bits(s, c2);

    if u8x32_highbit_any(s, c1) {
        Err(())
    } else {
        Ok(y)
    }
}
