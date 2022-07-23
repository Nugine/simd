use crate::error::Error;
use crate::format::{format_hyphenated_raw_fallback, format_hyphenated_raw_simd};
use crate::format::{format_simple_raw_fallback, format_simple_raw_simd};
use crate::parse::{parse_hyphenated_raw_fallback, parse_hyphenated_raw_simd};
use crate::parse::{parse_simple_raw_fallback, parse_simple_raw_simd};

use simd_abstraction::ascii::AsciiCase;
use simd_abstraction::simd_dispatch;

simd_dispatch!(
    name        = parse_simple_raw,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = parse_simple_raw_fallback,
    simd        = parse_simple_raw_simd,
    safety      = {unsafe},
);

simd_dispatch!(
    name        = parse_hyphenated_raw,
    signature   = fn(src: *const u8, dst: *mut u8) -> Result<(), Error>,
    fallback    = parse_hyphenated_raw_fallback,
    simd        = parse_hyphenated_raw_simd,
    safety      = {unsafe},
);

simd_dispatch!(
    name        = format_simple_raw,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = format_simple_raw_fallback,
    simd        = format_simple_raw_simd,
    safety      = {unsafe},
);

simd_dispatch!(
    name        = format_hyphenated_raw,
    signature   = fn(src: *const u8, dst: *mut u8, case: AsciiCase) -> (),
    fallback    = format_hyphenated_raw_fallback,
    simd        = format_hyphenated_raw_simd,
    safety      = {unsafe},
);
