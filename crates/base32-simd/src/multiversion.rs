use crate::{Error, Kind};

use vsimd::simd_dispatch;

simd_dispatch!(
    name        = check,
    signature   = fn(src: *const u8, len: usize, kind: Kind) -> Result<(), Error>,
    fallback    = {crate::check::check_fallback},
    simd        = {crate::check::check_simd},
    safety      = {unsafe},
    visibility  = {pub(crate)},
);

simd_dispatch!(
    name        = decode,
    signature   = fn(src: *const u8, len: usize, dst: *mut u8, kind: Kind) -> Result<(), Error>,
    fallback    = {crate::decode::decode_fallback},
    simd        = {crate::decode::decode_simd},
    safety      = {unsafe},
    visibility  = {pub(crate)},
);

simd_dispatch!(
    name        = encode,
    signature   = fn(src: *const u8, len: usize, dst: *mut u8, kind: Kind, padding: bool) -> (),
    fallback    = {crate::encode::encode_fallback},
    simd        = {crate::encode::encode_simd},
    safety      = {unsafe},
    visibility  = {pub(crate)},
);
