#![allow(missing_docs)]

vsimd::dispatch! (
    name        = {is_ascii},
    signature   = {pub unsafe fn(src: *const u8, len: usize) -> bool},
    fallback    = {crate::ascii::is_ascii_fallback},
    simd        = {crate::ascii::is_ascii_simd},
    targets     = {"avx2", "sse2", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);

vsimd::dispatch!(
    name        = {is_utf32le},
    signature   = {pub unsafe fn(src: *const u32, len: usize) -> bool},
    fallback    = {crate::utf32::is_utf32le_fallback},
    simd        = {crate::utf32::is_utf32le_simd},
    targets     = {"avx2", "sse4.1", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);

vsimd::dispatch!(
    name        = {utf32_swap_endianness},
    signature   = {pub unsafe fn(src: *const u32, len: usize, dst: *mut u32) -> ()},
    fallback    = {crate::utf32::swap_endianness_fallback},
    simd        = {crate::utf32::swap_endianness_simd},
    targets     = {"avx2", "ssse3", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);

vsimd::dispatch!(
    name        = {utf16_swap_endianness},
    signature   = {pub unsafe fn(src: *const u16, len: usize, dst: *mut u16) -> ()},
    fallback    = {crate::utf16::swap_endianness_fallback},
    simd        = {crate::utf16::swap_endianness_simd},
    targets     = {"avx2", "ssse3", "neon", "simd128"},
    fastest     = {"avx2", "neon", "simd128"},
);
