#![allow(missing_docs)]

use crate::error::Error;
use crate::Base64;

use simd_abstraction::simd_dispatch;

simd_dispatch!(
    name        = encode,
    signature   = fn(base64: &Base64, src: &[u8], dst: *mut u8) -> (),
    fallback    = {crate::encode::encode_fallback},
    simd        = {crate::encode::encode_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = decode,
    signature   = fn(base64: &Base64, n: usize, m: usize, src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::decode::decode_fallback},
    simd        = {crate::decode::decode_simd},
    safety      = {unsafe},
);
