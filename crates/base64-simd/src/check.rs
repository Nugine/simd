use crate::decode::{decode_ascii4, decode_ascii8, decode_extra};
use crate::decode::{STANDARD_DECODE_TABLE, URL_SAFE_DECODE_TABLE};
use crate::{Config, Error, Kind};

use vsimd::base64::{STANDARD_ALSW_CHECK_X2, URL_SAFE_ALSW_CHECK_X2};
use vsimd::tools::slice;
use vsimd::SIMD256;

use core::ptr::null_mut;

pub(crate) fn check_fallback(src: &[u8], config: Config) -> Result<(), Error> {
    let kind = config.kind;
    let forgiving = config.extra.forgiving();

    let (mut src, mut n) = (src.as_ptr(), src.len());

    let table = match kind {
        Kind::Standard => STANDARD_DECODE_TABLE.as_ptr(),
        Kind::UrlSafe => URL_SAFE_DECODE_TABLE.as_ptr(),
    };

    unsafe {
        // n*3/4 >= 6+2
        while n >= 11 {
            decode_ascii8::<false>(src, null_mut(), table)?;
            src = src.add(8);
            n -= 8;
        }

        while n >= 4 {
            decode_ascii4::<false>(src, null_mut(), table)?;
            src = src.add(4);
            n -= 4;
        }

        decode_extra::<false>(n, src, null_mut(), table, forgiving)
    }
}

pub(crate) fn check_simd<S: SIMD256>(s: S, src: &[u8], config: Config) -> Result<(), Error> {
    let kind = config.kind;

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

        check_fallback(slice(src, n), config)
    }
}
