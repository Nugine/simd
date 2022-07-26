#![allow(missing_docs)]

use crate::utf32::{is_utf32le_ct_fallback, is_utf32le_ct_simd};

use simd_abstraction::simd_dispatch;

simd_dispatch!(
    name        = is_utf32le_ct,
    signature   = fn(data: &[u32]) -> bool,
    fallback    = is_utf32le_ct_fallback,
    simd        = is_utf32le_ct_simd,
    safety      = {},
);
