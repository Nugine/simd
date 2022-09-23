use crate::fallback::{self, encode_bits24};
use crate::{Error, Kind};

use vsimd::base64::{STANDARD_ALSW_CHECK_X2, URL_SAFE_ALSW_CHECK_X2};
use vsimd::base64::{STANDARD_ALSW_DECODE_X2, URL_SAFE_ALSW_DECODE_X2};
use vsimd::base64::{STANDARD_CHARSET, URL_SAFE_CHARSET};

use vsimd::tools::{slice, slice_parts};
use vsimd::SIMD256;

pub(crate) unsafe fn encode<S: SIMD256>(s: S, src: &[u8], mut dst: *mut u8, kind: Kind, padding: bool) {
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

    fallback::encode(slice(src, len), dst, kind, padding)
}

pub unsafe fn decode<S: SIMD256>(
    s: S,
    mut src: *const u8,
    mut dst: *mut u8,
    mut n: usize,
    kind: Kind,
) -> Result<(), Error> {
    let (check_lut, decode_lut) = match kind {
        Kind::Standard => (STANDARD_ALSW_CHECK_X2, STANDARD_ALSW_DECODE_X2),
        Kind::UrlSafe => (URL_SAFE_ALSW_CHECK_X2, URL_SAFE_ALSW_DECODE_X2),
    };

    // n*3/4 >= 24+4
    while n >= 38 {
        let x = s.v256_load_unaligned(src);
        let y = vsimd::base64::decode_ascii32(s, x, check_lut, decode_lut).map_err(|()| Error::new())?;

        let (y1, y2) = y.to_v128x2();
        s.v128_store_unaligned(dst, y1);
        s.v128_store_unaligned(dst.add(12), y2);

        src = src.add(32);
        dst = dst.add(24);
        n -= 32;
    }

    fallback::decode(src, dst, n, kind)
}

pub fn check<S: SIMD256>(s: S, src: &[u8], kind: Kind) -> Result<(), Error> {
    let (mut src, mut n) = (src.as_ptr(), src.len());

    let check_lut = match kind {
        Kind::Standard => STANDARD_ALSW_CHECK_X2,
        Kind::UrlSafe => URL_SAFE_ALSW_CHECK_X2,
    };

    unsafe {
        // n*3/4 >= 24+4
        while n >= 38 {
            let x = s.v256_load_unaligned(src);
            let is_valid = vsimd::base64::check_ascii32(s, x, check_lut);
            ensure!(is_valid);
            src = src.add(32);
            n -= 32;
        }

        fallback::check(slice(src, n), kind)
    }
}
