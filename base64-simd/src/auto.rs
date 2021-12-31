use crate::fallback;
use crate::{Base64, Error, OutBuf};

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

impl Base64 {
    /// Encodes `src` and writes to `dst`.
    ///
    /// # Errors
    /// This function returns `Err` if:
    ///
    /// + The length of `dst` is not enough.
    pub fn encode<'s, 'd>(
        &'_ self,
        src: &'s [u8],
        dst: OutBuf<'d, u8>,
    ) -> Result<&'d mut [u8], Error> {
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
    pub fn decode<'s, 'd>(
        &'_ self,
        src: &'s [u8],
        dst: OutBuf<'d, u8>,
    ) -> Result<&'d mut [u8], Error> {
        try_simd!(decode(self, src, dst));
        fallback::decode(self, src, dst)
    }
}

#[test]
fn test() {
    crate::tests::test(Base64::encode, Base64::decode);
}
