use crate::{fallback, AsciiCase, Error, OutBuf};

macro_rules! try_simd {
    ($f:ident($($args:tt)*)) => {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            use simd_abstraction::traits::InstructionSet;
            use simd_abstraction::arch::x86::*;
            if AVX2::is_enabled() {
                return unsafe { $crate::arch::x86::avx2::$f($($args)*) };
            }
            if SSE41::is_enabled() {
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

            if NEON::is_enabled() {
                return unsafe { $crate::arch::arm::neon::$f($($args)*) };
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            use simd_abstraction::traits::InstructionSet;

            use simd_abstraction::arch::wasm::*;

            if SIMD128::is_enabled() {
                return unsafe { $crate::arch::wasm::simd128::$f($($args)*) };
            }
        }
    };
}

/// Checks whether `src` is a hex string.
#[inline]
pub fn check(src: &[u8]) -> bool {
    try_simd!(check(src));
    fallback::check(src)
}

/// Encodes `src` with a given ascii case and writes to `dst`.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `dst` is not enough.
///
#[inline]
pub fn encode<'s, 'd>(
    src: &'s [u8],
    dst: OutBuf<'d>,
    case: AsciiCase,
) -> Result<&'d mut [u8], Error> {
    try_simd!(encode(src, dst, case));
    fallback::encode(src, dst, case)
}

/// Decodes `src` case-insensitively and writes to `dst`.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `dst` is not enough.
/// + The content of `src` is invalid.
///
#[inline]
pub fn decode<'s, 'd>(src: &'s [u8], dst: OutBuf<'d>) -> Result<&'d mut [u8], Error> {
    try_simd!(decode(src, dst));
    fallback::decode(src, dst)
}

/// Decodes `buf` case-insensitively and writes inplace.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The content of `buf` is invalid.
///
/// When this function returns `Err`, the content of `buf` should be considered as fully broken.
///
#[inline]
pub fn decode_inplace(buf: &mut [u8]) -> Result<&mut [u8], Error> {
    try_simd!(decode_inplace(buf));
    fallback::decode_inplace(buf)
}
