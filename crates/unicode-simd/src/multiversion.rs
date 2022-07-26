#![allow(missing_docs)]

use simd_abstraction::simd_dispatch;

simd_dispatch!(
    name        = is_utf32le_ct,
    signature   = fn(data: &[u32]) -> bool,
    fallback    = {crate::utf32::is_utf32le_ct_fallback},
    simd        = {crate::utf32::is_utf32le_ct_simd},
    safety      = {},
);
