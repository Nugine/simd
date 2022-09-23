use crate::decode::{decode_bits, BASE32HEX_TABLE, BASE32_TABLE};
use crate::Error;

use vsimd::base32::Kind;
use vsimd::base32::{BASE32HEX_ALSW_CHECK_X2, BASE32_ALSW_CHECK_X2};
use vsimd::tools::{slice, slice_parts};
use vsimd::SIMD256;

#[inline]
unsafe fn check_extra(src: *const u8, extra: usize, table: *const u8) -> Result<(), Error> {
    match extra {
        0 => {}
        2 => {
            let (u10, flag) = decode_bits::<2>(src, table);
            ensure!(flag != 0xff && u10 & 0b11 == 0);
        }
        4 => {
            let (u20, flag) = decode_bits::<4>(src, table);
            ensure!(flag != 0xff && u20 & 0b1111 == 0);
        }
        5 => {
            let (u25, flag) = decode_bits::<5>(src, table);
            ensure!(flag != 0xff && u25 & 0b1 == 0);
        }
        7 => {
            let (u35, flag) = decode_bits::<7>(src, table);
            ensure!(flag != 0xff && u35 & 0b111 == 0);
        }
        _ => core::hint::unreachable_unchecked(),
    }
    Ok(())
}

#[inline]
pub fn check_fallback(src: &[u8], kind: Kind) -> Result<(), Error> {
    let table = match kind {
        Kind::Base32 => BASE32_TABLE.as_ptr(),
        Kind::Base32Hex => BASE32HEX_TABLE.as_ptr(),
    };

    unsafe {
        let (mut src, mut len) = slice_parts(src);

        let end = src.add(len / 8 * 8);
        while src < end {
            let (_, flag) = decode_bits::<8>(src, table);
            ensure!(flag != 0xff);
            src = src.add(8);
        }
        len %= 8;

        check_extra(src, len, table)
    }
}

#[inline]
pub fn check_simd<S: SIMD256>(s: S, src: &[u8], kind: Kind) -> Result<(), Error> {
    let check_lut = match kind {
        Kind::Base32 => BASE32_ALSW_CHECK_X2,
        Kind::Base32Hex => BASE32HEX_ALSW_CHECK_X2,
    };

    unsafe {
        let (mut src, mut len) = slice_parts(src);

        let end = src.add(len / 32 * 32);
        while src < end {
            let x = s.v256_load_unaligned(src);

            let is_valid = vsimd::base32::check_ascii32(s, x, check_lut);
            ensure!(is_valid);

            src = src.add(32);
        }
        len %= 32;

        check_fallback(slice(src, len), kind)
    }
}
