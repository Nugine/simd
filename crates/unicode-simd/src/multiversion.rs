#![allow(missing_docs)]

use simd_abstraction::simd_dispatch;

simd_dispatch!(
    name        = is_utf32le_ct,
    signature   = fn(data: &[u32]) -> bool,
    fallback    = {crate::utf32::is_utf32le_ct_fallback},
    simd        = {crate::utf32::is_utf32le_ct_simd},
    safety      = {},
);

simd_dispatch!(
    name        = utf32_swap_endianness,
    signature   = fn(src: *const u32, len: usize, dst: *mut u32) -> (),
    fallback    = {crate::utf32::utf32_swap_endianness_fallback},
    simd        = {crate::utf32::utf32_swap_endianness_simd},
    safety      = {unsafe},
);

simd_dispatch!(
    name        = utf16_swap_endianness,
    signature   = fn(src: *const u16, len: usize, dst: *mut u16) -> (),
    fallback    = {crate::utf16::utf16_swap_endianness_fallback},
    simd        = {crate::utf16::utf16_swap_endianness_simd},
    safety      = {unsafe},
);
