use crate::fallback::{self, encode_bits, read_be_bytes};
use crate::Error;

use vsimd::base32::Kind;
use vsimd::base32::{BASE32HEX_ALSW_CHECK_X2, BASE32HEX_ALSW_DECODE_X2};
use vsimd::base32::{BASE32HEX_CHARSET, BASE32_CHARSET};
use vsimd::base32::{BASE32HEX_ENCODING_LUT, BASE32_ENCODING_LUT};
use vsimd::base32::{BASE32_ALSW_CHECK_X2, BASE32_ALSW_DECODE_X2};
use vsimd::tools::{slice, slice_parts};
use vsimd::SIMD256;

#[inline(always)]
pub fn check<S: SIMD256>(s: S, src: &[u8], kind: Kind) -> Result<(), Error> {
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

        fallback::check(slice(src, len), kind)
    }
}

#[inline(always)]
pub unsafe fn encode<S: SIMD256>(s: S, src: &[u8], mut dst: *mut u8, kind: Kind, padding: bool) {
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

    fallback::encode(slice(src, len), dst, kind, padding);
}

#[inline(always)]
pub unsafe fn decode<S: SIMD256>(
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
        let y = try_!(vsimd::base32::decode_ascii32(s, x, check_lut, decode_lut));

        let (y1, y2) = y.to_v128x2();
        s.v128_store_unaligned(dst, y1);
        s.v128_store_unaligned(dst.add(10), y2);

        src = src.add(32);
        dst = dst.add(20);
        n -= 32;
    }

    fallback::decode(src, n, dst, kind)
}
