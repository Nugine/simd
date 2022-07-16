#![allow(missing_docs)]

use crate::Error;

use crate::check::{check_fallback, check_simd};
use crate::decode::{decode_raw_fallback, decode_raw_simd};
use crate::encode::{encode_raw_fallback, encode_raw_simd};

use simd_abstraction::ascii::AsciiCase;
use simd_abstraction::simd_dispatch;

simd_dispatch!(
    name        = check,
    signature   = fn(data: &[u8]) -> bool,
    fallback    = check_fallback,
    simd        = check_simd,
    safety      = {},
);

simd_dispatch!(
    name        = encode_raw,
    signature   = fn(src: &[u8], dst: *mut u8, case: AsciiCase) -> (),
    fallback    = encode_raw_fallback,
    simd        = encode_raw_simd,
    safety      = {unsafe},
);

simd_dispatch!(
    name        = decode_raw,
    signature   = fn(src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error>,
    fallback    = decode_raw_fallback,
    simd        = decode_raw_simd,
    safety      = {unsafe},
);
