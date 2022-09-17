use super::*;

use vsimd::simd_dispatch;

simd_dispatch!(
    name        = encode,
    signature   = fn(src: &[u8], dst: *mut u8, kind: Kind, padding: bool) -> (),
    fallback    = {crate::encode::encode_fallback},
    simd        = {crate::encode::encode_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = decode,
    signature   = fn(src: *const u8, dst: *mut u8, n: usize, kind: Kind) -> Result<(), Error>,
    fallback    = {crate::decode::decode_fallback},
    simd        = {crate::decode::decode_simd},
    safety      = {unsafe},
);
