#![allow(missing_docs)]

use crate::{AsciiCase, Error};

vsimd::dispatch!(
    name        = {check},
    signature   = {pub unsafe fn(src: *const u8, len: usize) -> Result<(), Error>},
    fallback    = {crate::check::check_fallback},
    simd        = {crate::check::check_simd},
    targets     = {"avx2", "sse2", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);

vsimd::dispatch!(
    name        = {encode},
    signature   = {pub unsafe fn(src: *const u8, len: usize, dst: *mut u8, case: AsciiCase) -> () },
    fallback    = {crate::encode::encode_fallback},
    simd        = {crate::encode::encode_simd},
    targets     = {"avx2", "ssse3", "sse2", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);

vsimd::dispatch!(
    name        = {decode},
    signature   = {pub unsafe fn(src: *const u8, len: usize, dst: *mut u8) -> Result<(), Error>},
    fallback    = {crate::decode::decode_fallback},
    simd        = {crate::decode::decode_simd},
    targets     = {"avx2", "ssse3", "sse2", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);
