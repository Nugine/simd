use crate::{Config, Error};

use vsimd::simd_dispatch;

simd_dispatch!(
    name        = encode,
    signature   = fn(src: &[u8], dst: *mut u8, config: Config) -> (),
    fallback    = {crate::fallback::encode},
    simd        = {crate::simd::encode},
    safety      = {unsafe},
    visibility  = {pub(crate)},
);

simd_dispatch!(
    name        = decode,
    signature   = fn(src: *const u8, dst: *mut u8, n: usize, config: Config) -> Result<(), Error>,
    fallback    = {crate::fallback::decode},
    simd        = {crate::simd::decode},
    safety      = {unsafe},
    visibility  = {pub(crate)},
);

simd_dispatch!(
    name        = check,
    signature   = fn(src: &[u8], config: Config) -> Result<(), Error>,
    fallback    = {crate::fallback::check},
    simd        = {crate::simd::check},
    safety      = {},
    visibility  = {pub(crate)},
);
