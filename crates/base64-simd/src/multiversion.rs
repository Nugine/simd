use super::*;

use vsimd::simd_dispatch;

simd_dispatch!(
    name        = encode,
    signature   = fn(src: &[u8], dst: *mut u8, kind: Kind, padding: bool) -> (),
    fallback    = {crate::fallback::encode},
    simd        = {crate::simd::encode},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = decode,
    signature   = fn(src: *const u8, dst: *mut u8, n: usize, kind: Kind) -> Result<(), Error>,
    fallback    = {crate::fallback::decode},
    simd        = {crate::simd::decode},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = check,
    signature   = fn(src: &[u8], kind: Kind) -> Result<(), Error>,
    fallback    = {crate::fallback::check},
    simd        = {crate::simd::check},
    safety      = {},
);
