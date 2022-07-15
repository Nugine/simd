use crate::fallback;
use crate::{Base64, Error, OutBuf};

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

impl Base64 {
    /// Encodes `src` and writes to `dst`.
    ///
    /// # Errors
    /// This function returns `Err` if:
    ///
    /// + The length of `dst` is not enough.
    #[inline]
    pub fn encode<'s, 'd>(&'_ self, src: &'s [u8], dst: OutBuf<'d>) -> Result<&'d mut [u8], Error> {
        try_simd!(encode(self, src, dst));
        fallback::encode(self, src, dst)
    }

    /// Decodes `src` and writes to `dst`.
    ///
    /// # Errors
    /// This function returns `Err` if:
    ///
    /// + The length of `dst` is not enough.
    /// + The content of `src` is invalid.
    #[inline]
    pub fn decode<'s, 'd>(&'_ self, src: &'s [u8], dst: OutBuf<'d>) -> Result<&'d mut [u8], Error> {
        try_simd!(decode(self, src, dst));
        fallback::decode(self, src, dst)
    }

    /// Decodes `buf` and writes inplace.
    ///
    /// # Errors
    /// This function returns `Err` if:
    ///
    /// + The content of `buf` is invalid.
    ///
    /// When this function returns `Err`, the content of `buf` should be considered as fully broken.
    #[inline]
    pub fn decode_inplace<'b>(&'_ self, buf: &'b mut [u8]) -> Result<&'b mut [u8], Error> {
        try_simd!(decode_inplace(self, buf));
        fallback::decode_inplace(self, buf)
    }
}
