#![allow(missing_docs)]

use crate::sa_ascii::AsciiCase;
use crate::Error;

use simd_abstraction::simd_dispatch;

simd_dispatch!(
    name        = check,
    signature   = fn(data: &[u8]) -> bool,
    fallback    = {crate::check::check_fallback},
    simd        = {crate::check::check_simd},
    safety      = {},
);

simd_dispatch!(
    name        = encode,
    signature   = fn(src: &[u8], dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::encode::encode_fallback},
    simd        = {crate::encode::encode_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = decode,
    signature   = fn(src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::decode::decode_fallback},
    simd        = {crate::decode::decode_simd},
    safety      = {unsafe},
);
