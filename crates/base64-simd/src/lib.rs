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

mod error;
pub use self::error::Error;

mod spec;

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

    /// Calculates the encoded length.
    ///
    /// # Panics
    /// This function panics if any of the conditions below is not satisfied:
    ///
    /// + `n <= isize::MAX`
    #[inline]
    pub const fn encoded_length(&self, n: usize) -> usize {
        assert!(n < usize::MAX / 2);
        unsafe { self::encode::encoded_length_unchecked(n, self.padding) }
    }

    /// Estimates the decoded length.
    ///
    /// The result is an upper bound which can be used for allocation.
    #[inline]
    pub const fn estimated_decoded_length(&self, n: usize) -> usize {
        if n % 4 == 0 {
            n / 4 * 3
        } else {
            (n / 4 + 1) * 3
        }
    }

    /// Calculates the decoded length.
    ///
    /// The result is a precise value which can be used for allocation.
    #[inline]
    pub fn decoded_length(&self, data: &[u8]) -> Result<usize, Error> {
        let (_, m) = self::decode::decoded_length(data, self.padding)?;
        Ok(m)
    }
}
