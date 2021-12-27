use crate::{AsciiCase, Error, Hex};

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
    };
}

#[inline]
pub fn parse(src: &[u8]) -> Result<[u8; 16], Error> {
    try_simd!(parse(src));
    crate::fallback::parse(src)
}

#[inline]
pub fn parse_simple(src: &[u8]) -> Result<[u8; 16], Error> {
    try_simd!(parse_simple(src));
    crate::fallback::parse_simple(src)
}

#[inline]
pub fn parse_hyphenated(src: &[u8]) -> Result<[u8; 16], Error> {
    try_simd!(parse_hyphenated(src));
    crate::fallback::parse_hyphenated(src)
}

#[inline]
pub fn format_simple(src: &[u8; 16], case: AsciiCase) -> Hex<32> {
    try_simd!(format_simple(src, case));
    crate::fallback::format_simple(src, case)
}

#[inline]
pub fn format_hyphenated(src: &[u8; 16], case: AsciiCase) -> Hex<36> {
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
