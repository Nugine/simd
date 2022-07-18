// // ! SIMD-accelerated base64 encoding and decoding.
// // !
// // ! # Examples
// // !
// // ! ```
// // ! use base64_simd::Base64;
// // !
// // ! let bytes = b"hello world";
// // ! let base64 = Base64::STANDARD;
// // !
// // ! let encoded = base64.encode_to_boxed_str(bytes);
// // ! assert_eq!(&*encoded, "aGVsbG8gd29ybGQ=");
// // !
// // ! let decoded = base64.decode_to_boxed_bytes(encoded.as_bytes()).unwrap();
// // ! assert_eq!(&*decoded, bytes);
// // ! ```
// // !

//! TODO
// TODO:

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
//
#![deny(
    missing_debug_implementations,
    missing_docs,
    clippy::all,
    clippy::cargo,
    clippy::missing_inline_in_public_items
)]
#![warn(clippy::todo)]

#[cfg(feature = "alloc")]
extern crate alloc;

// pub(crate) use simd_abstraction::ascii as sa_ascii;

// pub use simd_abstraction::tools::OutBuf;

// pub(crate) use self::error::ERROR;

// #[cfg(test)]
// mod tests;

// pub mod fallback;

// #[macro_use]
// mod generic;

// mod polyfill;

// pub mod arch;

// mod auto;

// mod ext;

// impl Base64 {
//     const PAD: u8 = b'=';

//     #[inline(always)]
//     const unsafe fn encoded_length_unchecked(n: usize, padding: bool) -> usize {
//         let extra = n % 3;
//         if extra == 0 {
//             n / 3 * 4
//         } else if padding {
//             n / 3 * 4 + 4
//         } else {
//             n / 3 * 4 + extra + 1
//         }
//     }

//     /// # Safety
//     /// This function requires:
//     ///
//     /// + `src.len() > 0`
//     #[inline(always)]
//     unsafe fn decoded_length_unchecked(src: &[u8], padding: bool) -> Result<(usize, usize), Error> {
//         let n = {
//             let len = src.len();
//             if padding {
//                 if len % 4 != 0 {
//                     return Err(ERROR);
//                 }
//                 let last1 = *src.get_unchecked(len - 1);
//                 let last2 = *src.get_unchecked(len - 2);
//                 let count = (last1 == Base64::PAD) as usize + (last2 == Base64::PAD) as usize;
//                 len - count
//             } else {
//                 len
//             }
//         };

//         let m = match n % 4 {
//             0 => n / 4 * 3,
//             1 => return Err(ERROR),
//             2 => n / 4 * 3 + 1,
//             3 => n / 4 * 3 + 2,
//             _ => core::hint::unreachable_unchecked(),
//         };

//         Ok((n, m))
//     }

//     /// Calcuates the encoding length.
//     ///
//     /// # Panics
//     /// This function panics if any of the conditions below is not satisfied:
//     ///
//     /// + `n <= isize::MAX`
//     #[inline]
//     pub const fn encoded_length(&self, n: usize) -> usize {
//         assert!(n <= (isize::MAX as usize));
//         unsafe { Self::encoded_length_unchecked(n, self.padding) }
//     }

//     /// Returns the character set used for encoding.
//     #[inline]
//     pub const fn charset(&self) -> &[u8; 64] {
//         match self.kind {
//             Base64Kind::Standard => fallback::STANDARD_CHARSET,
//             Base64Kind::UrlSafe => fallback::URL_SAFE_CHARSET,
//         }
//     }
// }

mod error;
pub use self::error::Error;

mod decode;
mod encode;

pub mod multiversion;

// -------------------------------------------------------------------------------------------------

#[derive(Debug)]
enum Base64Kind {
    Standard,
    UrlSafe,
}

const STANDARD_CHARSET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const URL_SAFE_CHARSET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Base64 variants
///
/// + [`Base64::STANDARD`](crate::Base64::STANDARD)
/// + [`Base64::STANDARD_NO_PAD`](crate::Base64::STANDARD_NO_PAD)
/// + [`Base64::URL_SAFE`](crate::Base64::URL_SAFE)
/// + [`Base64::URL_SAFE_NO_PAD`](crate::Base64::URL_SAFE_NO_PAD)
///
#[derive(Debug)]
pub struct Base64 {
    kind: Base64Kind,
    padding: bool,
}

impl Base64 {
    /// Standard charset with padding.
    pub const STANDARD: Self = Self {
        kind: Base64Kind::Standard,
        padding: true,
    };

    /// Standard charset without padding.
    pub const STANDARD_NO_PAD: Self = Self {
        kind: Base64Kind::Standard,
        padding: false,
    };

    /// URL-safe charset with padding.
    pub const URL_SAFE: Self = Self {
        kind: Base64Kind::UrlSafe,
        padding: true,
    };

    /// URL-safe charset without padding.
    pub const URL_SAFE_NO_PAD: Self = Self {
        kind: Base64Kind::UrlSafe,
        padding: false,
    };

    /// Returns the character set used for encoding.
    #[inline]
    pub const fn charset(&self) -> &[u8; 64] {
        match self.kind {
            Base64Kind::Standard => STANDARD_CHARSET,
            Base64Kind::UrlSafe => URL_SAFE_CHARSET,
        }
    }
}
