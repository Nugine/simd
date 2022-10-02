use crate::error::Error;

use vsimd::ascii::AsciiCase;
use vsimd::simd_dispatch;

simd_dispatch!(
    name        = parse_simple,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::fallback::parse_simple},
    simd        = {crate::simd::parse_simple},
    safety      = {unsafe},
    visibility  = {pub},
);

simd_dispatch!(
    name        = parse_hyphenated,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::fallback::parse_hyphenated},
    simd        = {crate::simd::parse_hyphenated},
    safety      = {unsafe},
    visibility  = {pub},
);

simd_dispatch!(
    name        = format_simple,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::fallback::format_simple},
    simd        = {crate::simd::format_simple},
    safety      = {unsafe},
    visibility  = {pub},
);

simd_dispatch!(
    name        = format_hyphenated,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::fallback::format_hyphenated},
    simd        = {crate::simd::format_hyphenated},
    safety      = {unsafe},
    visibility  = {pub},
);
