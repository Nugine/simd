#![allow(missing_docs)]

use crate::check::{check_fallback, check_simd};

use simd_abstraction::simd_dispatch;

simd_dispatch!(
    name        = check,
    signature   = fn(data: &[u8]) -> bool,
    fallback    = check_fallback,
    simd        = check_simd,
    safety      = {},
);
