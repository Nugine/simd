#![allow(missing_docs)]

use vsimd::simd_dispatch;

simd_dispatch!(
    name        = is_utf32le_ct,
    signature   = fn(data: &[u32]) -> bool,
    fallback    = {crate::fallback::utf32::is_utf32le_ct},
    simd        = {crate::simd::utf32::is_utf32le_ct},
    safety      = {},
);

simd_dispatch!(
    name        = utf32_swap_endianness,
    signature   = fn(src: *const u32, len: usize, dst: *mut u32) -> (),
    fallback    = {crate::fallback::utf32::swap_endianness},
    simd        = {crate::simd::utf32::swap_endianness},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = utf16_swap_endianness,
    signature   = fn(src: *const u16, len: usize, dst: *mut u16) -> (),
    fallback    = {crate::fallback::utf16::swap_endianness},
    simd        = {crate::simd::utf16::swap_endianness},
    safety      = {unsafe},
);
