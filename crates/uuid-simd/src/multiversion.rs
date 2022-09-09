use crate::error::Error;

use vsimd::ascii::AsciiCase;
use vsimd::simd_dispatch;

simd_dispatch!(
    name        = parse_simple_raw,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::parse::parse_simple_fallback},
    simd        = {crate::parse::parse_simple_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = parse_hyphenated_raw,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::parse::parse_hyphenated_fallback},
    simd        = {crate::parse::parse_hyphenated_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = format_simple_raw,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::format::format_simple_fallback},
    simd        = {crate::format::format_simple_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = format_hyphenated_raw,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::format::format_hyphenated_fallback},
    simd        = {crate::format::format_hyphenated_simd},
    safety      = {unsafe},
);
