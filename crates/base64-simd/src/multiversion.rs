use crate::{Config, Error};

use vsimd::simd_dispatch;

simd_dispatch!(
    name        = encode,
    signature   = fn(src: *const u8, len: usize, dst: *mut u8, config: Config) -> (),
    fallback    = {crate::encode::encode_fallback},
    simd        = {crate::encode::encode_simd},
    safety      = {unsafe},
    visibility  = {pub(crate)},
);

simd_dispatch!(
    name        = decode,
    signature   = fn(src: *const u8, dst: *mut u8, n: usize, config: Config) -> Result<(), Error>,
    fallback    = {crate::decode::decode_fallback},
    simd        = {crate::decode::decode_simd},
    safety      = {unsafe},
    visibility  = {pub(crate)},
);

simd_dispatch!(
    name        = check,
    signature   = fn(src: *const u8, n: usize, config: Config) -> Result<(), Error>,
    fallback    = {crate::check::check_fallback},
    simd        = {crate::check::check_simd},
    safety      = {unsafe},
    visibility  = {pub(crate)},
);

simd_dispatch!(
    name        = find_non_ascii_whitespace,
    signature   = fn(src: *const u8, len: usize) -> usize,
    fallback    = {crate::ascii::find_non_ascii_whitespace_fallback},
    simd        = {crate::ascii::find_non_ascii_whitespace_simd},
    safety      = {unsafe},
    visibility  = {pub},
);
