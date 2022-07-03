use crate::{AsciiCase, Error, HexStr};

macro_rules! try_simd {
    ($f:ident($($args:tt)*)) => {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            use simd_abstraction::traits::InstructionSet;
            use simd_abstraction::arch::x86::*;
            if AVX2::detect().is_some() {
                return unsafe { $crate::arch::x86::avx2::$f($($args)*) };
            }
            if SSE41::detect().is_some() {
                return unsafe { $crate::arch::x86::sse41::$f($($args)*) };
            }
        }
        #[cfg(all(
            feature="unstable",
            any(target_arch = "arm", target_arch="aarch64")
        ))]
        {
            use simd_abstraction::traits::InstructionSet;

            #[cfg(target_arch="arm")]
            use simd_abstraction::arch::arm::*;

            #[cfg(target_arch="aarch64")]
            use simd_abstraction::arch::aarch64::*;

            if NEON::detect().is_some() {
                return unsafe { $crate::arch::arm::neon::$f($($args)*) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            use simd_abstraction::traits::InstructionSet;

            use simd_abstraction::arch::wasm::*;

            if SIMD128::detect().is_some() {
                return unsafe { $crate::arch::wasm::simd128::$f($($args)*) };
            }
        }
    };
}

/// Parses an UUID from arbitrary bytes.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `src` doesn't match any UUID format variants.
/// + The content of `src` is invalid.
///
#[inline]
pub fn parse(src: &[u8]) -> Result<[u8; 16], Error> {
    try_simd!(parse(src));
    crate::fallback::parse(src)
}

/// Parses a simple UUID from arbitrary bytes.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `src` doesn't match the "simple" format.
/// + The content of `src` is invalid.
///
#[inline]
pub fn parse_simple(src: &[u8]) -> Result<[u8; 16], Error> {
    try_simd!(parse_simple(src));
    crate::fallback::parse_simple(src)
}

/// Parses a hyphenated UUID from arbitrary bytes.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `src` doesn't match the "hyphenated" format.
/// + The content of `src` is invalid.
///
#[inline]
pub fn parse_hyphenated(src: &[u8]) -> Result<[u8; 16], Error> {
    try_simd!(parse_hyphenated(src));
    crate::fallback::parse_hyphenated(src)
}

/// Formats `src` to a fixed length hex string.
#[inline]
pub fn format_simple(src: &[u8; 16], case: AsciiCase) -> HexStr<32> {
    try_simd!(format_simple(src, case));
    crate::fallback::format_simple(src, case)
}

/// Formats `src` to a fixed length hex string.
#[inline]
pub fn format_hyphenated(src: &[u8; 16], case: AsciiCase) -> HexStr<36> {
    try_simd!(format_hyphenated(src, case));
    crate::fallback::format_hyphenated(src, case)
}

#[test]
fn test_parse() {
    crate::tests::test_parse_ok(|s| parse(s.as_bytes()));
    crate::tests::test_parse_err(|s| parse(s.as_bytes()));
}

#[test]
fn test_format() {
    crate::tests::test_format_simple(format_simple);
    crate::tests::test_format_hypenated(format_hyphenated);
}
