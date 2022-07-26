use crate::error::Error;

use simd_abstraction::ascii::AsciiCase;
use simd_abstraction::simd_dispatch;

simd_dispatch!(
    name        = parse_simple_raw,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::parse::parse_simple_raw_fallback},
    simd        = {crate::parse::parse_simple_raw_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = parse_hyphenated_raw,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = {crate::parse::parse_hyphenated_raw_fallback},
    simd        = {crate::parse::parse_hyphenated_raw_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = format_simple_raw,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::format::format_simple_raw_fallback},
    simd        = {crate::format::format_simple_raw_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = format_hyphenated_raw,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = {crate::format::format_hyphenated_raw_fallback},
    simd        = {crate::format::format_hyphenated_raw_simd},
    safety      = {unsafe},
);
