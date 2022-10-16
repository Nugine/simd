use crate::error::Error;

use vsimd::ascii::AsciiCase;
use vsimd::simd_dispatch;

simd_dispatch!(
    name        = parse_simple,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::parse::parse_simple_fallback},
    simd        = {crate::parse::parse_simple_simd},
    safety      = {unsafe},
    visibility  = {pub},
);

simd_dispatch!(
    name        = parse_hyphenated,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::parse::parse_hyphenated_fallback},
    simd        = {crate::parse::parse_hyphenated_simd},
    safety      = {unsafe},
    visibility  = {pub},
);

simd_dispatch!(
    name        = format_simple,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::format::format_simple_fallback},
    simd        = {crate::format::format_simple_simd},
    safety      = {unsafe},
    visibility  = {pub},
);

simd_dispatch!(
    name        = format_hyphenated,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::format::format_hyphenated_fallback},
    simd        = {crate::format::format_hyphenated_simd},
    safety      = {unsafe},
    visibility  = {pub},
);
