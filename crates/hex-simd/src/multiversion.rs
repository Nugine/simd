#![allow(missing_docs)]

use crate::{AsciiCase, Error};

use vsimd::simd_dispatch;

simd_dispatch!(
    name        = check,
    signature   = fn(data: &[u8]) -> Result<(), Error>,
    fallback    = {crate::fallback::check},
    simd        = {crate::simd::check},
    safety      = {},
    visibility  = {pub},
);

simd_dispatch!(
    name        = encode,
    signature   = fn(src: &[u8], dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::fallback::encode},
    simd        = {crate::simd::encode},
    safety      = {unsafe},
    visibility  = {pub},
);

simd_dispatch!(
    name        = decode,
    signature   = fn(src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::fallback::decode},
    simd        = {crate::simd::decode},
    safety      = {unsafe},
    visibility  = {pub},
);
