//! SIMD-accelerated base64 encoding and decoding.
//!
//! # Examples
//!
//! ```
//! let bytes = b"hello world";
//! let base64 = base64_simd::STANDARD;
//!
//! let encoded = base64.encode_type::<String>(bytes);
//! assert_eq!(&*encoded, "aGVsbG8gd29ybGQ=");
//!
//! let decoded = base64.decode_type::<Vec<u8>>(encoded.as_bytes()).unwrap();
//! assert_eq!(&*decoded, bytes);
//! ```
//!

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
#![cfg_attr(docsrs, feature(doc_cfg))]
//
#![deny(
    missing_debug_implementations,
    missing_docs,
    clippy::all,
    clippy::cargo,
    clippy::missing_inline_in_public_items,
    clippy::must_use_candidate
)]
#![warn(clippy::todo)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod error;
pub use self::error::Error;

mod check;
mod decode;
mod encode;
mod forgiving;

mod multiversion;

#[cfg(feature = "alloc")]
mod heap;

#[cfg(test)]
mod tests;

pub use outref::OutRef;

// -----------------------------------------------------------------------------

use crate::decode::decoded_length;
use crate::encode::encoded_length_unchecked;

use vsimd::base64::Kind;
use vsimd::base64::{STANDARD_CHARSET, URL_SAFE_CHARSET};
use vsimd::tools::slice_mut;

/// Base64 variant
#[derive(Debug)]
pub struct Base64 {
    kind: Kind,
    padding: bool,
}

/// Standard charset with padding.
pub const STANDARD: Base64 = Base64 {
    kind: Kind::Standard,
    padding: true,
};

/// URL-Safe charset with padding.
pub const URL_SAFE: Base64 = Base64 {
    kind: Kind::UrlSafe,
    padding: true,
};

/// Standard charset without padding.
pub const STANDARD_NO_PAD: Base64 = Base64 {
    kind: Kind::Standard,
    padding: false,
};

/// URL-Safe charset without padding.
pub const URL_SAFE_NO_PAD: Base64 = Base64 {
    kind: Kind::UrlSafe,
    padding: false,
};

impl Base64 {
    /// Returns the character set.
    #[inline]
    #[must_use]
    pub const fn charset(&self) -> &[u8; 64] {
        match self.kind {
            Kind::Standard => STANDARD_CHARSET,
            Kind::UrlSafe => URL_SAFE_CHARSET,
        }
    }

    /// Calculates the encoded length.
    ///
    /// # Panics
    /// This function will panic if `n > isize::MAX`.
    #[inline]
    #[must_use]
    pub const fn encoded_length(&self, n: usize) -> usize {
        assert!(n <= usize::MAX / 2);
        encoded_length_unchecked(n, self.padding)
    }

    /// Estimates the decoded length.
    ///
    /// The result is an upper bound which can be used for allocation.
    #[inline]
    #[must_use]
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
        let (_, m) = decoded_length(data, self.padding)?;
        Ok(m)
    }

    /// Checks whether `data` is a base64 string.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    #[inline]
    pub fn check(&self, data: &[u8]) -> Result<(), Error> {
        let (n, _) = decoded_length(data, self.padding)?;
        let src = unsafe { data.get_unchecked(..n) };
        crate::multiversion::check::auto(src, self.kind)
    }

    /// Encodes bytes to a base64 string.
    ///
    /// # Panics
    /// This function will panic if the length of `dst` is not enough.
    #[inline]
    #[must_use]
    pub fn encode<'s, 'd>(&'_ self, src: &'s [u8], mut dst: OutRef<'d, [u8]>) -> &'d mut [u8] {
        unsafe {
            let m = encoded_length_unchecked(src.len(), self.padding);
            assert!(dst.len() >= m);

            let dst = dst.as_mut_ptr();
            self::multiversion::encode::auto(src, dst, self.kind, self.padding);

            slice_mut(dst, m)
        }
    }

    /// Encodes bytes to a base64 string and returns [`&mut str`](str).
    ///
    /// # Panics
    /// This function will panic if the length of `dst` is not enough.
    #[inline]
    #[must_use]
    pub fn encode_as_str<'s, 'd>(&'_ self, src: &'s [u8], dst: OutRef<'d, [u8]>) -> &'d mut str {
        let ans = self.encode(src, dst);
        unsafe { core::str::from_utf8_unchecked_mut(ans) }
    }

    /// Decodes a base64 string to bytes.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `src` is invalid.
    ///
    /// # Panics
    /// This function will panic if the length of `dst` is not enough.
    #[inline]
    pub fn decode<'s, 'd>(&'_ self, src: &'s [u8], mut dst: OutRef<'d, [u8]>) -> Result<&'d mut [u8], Error> {
        unsafe {
            let (n, m) = decoded_length(src, self.padding)?;
            assert!(dst.len() >= m);

            let src = src.as_ptr();
            let dst = dst.as_mut_ptr();
            self::multiversion::decode::auto(src, dst, n, self.kind)?;

            Ok(slice_mut(dst, m))
        }
    }

    /// Decodes a base64 string to bytes and writes inplace.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    #[inline]
    pub fn decode_inplace<'d>(&'_ self, data: &'d mut [u8]) -> Result<&'d mut [u8], Error> {
        unsafe {
            let (n, m) = decoded_length(data, self.padding)?;

            let dst: *mut u8 = data.as_mut_ptr();
            let src: *const u8 = dst;
            self::multiversion::decode::auto(src, dst, n, self.kind)?;

            Ok(slice_mut(dst, m))
        }
    }

    /// Encodes bytes to a base64 string and returns a specified type.
    #[inline]
    #[must_use]
    pub fn encode_type<T: FromBase64Encode>(&self, data: &[u8]) -> T {
        T::from_base64_encode(self, data)
    }

    /// Decodes a base64 string to bytes and returns a specified type.
    #[inline]
    pub fn decode_type<T: FromBase64Decode>(&self, data: &[u8]) -> Result<T, Error> {
        T::from_base64_decode(self, data)
    }
}

/// Forgiving decodes a base64 string to bytes and writes inplace.
///
/// This function uses the standard charset.
///
/// See <https://infra.spec.whatwg.org/#forgiving-base64>
///
/// # Errors
/// This function returns `Err` if the content of `data` is invalid.
#[inline]
pub fn forgiving_decode_inplace(data: &mut [u8]) -> Result<&mut [u8], Error> {
    let data = crate::forgiving::normalize(data);
    STANDARD_NO_PAD.decode_inplace(data)
}

/// Types that can represent a base64 string.
pub trait FromBase64Encode: Sized {
    /// Encodes bytes to a base64 string and returns the self type.
    fn from_base64_encode(base64: &Base64, data: &[u8]) -> Self;
}

/// Types that can be decoded from a base64 string.
pub trait FromBase64Decode: Sized {
    /// Decodes a base64 string to bytes case-insensitively and returns the self type.
    fn from_base64_decode(base64: &Base64, data: &[u8]) -> Result<Self, Error>;
}
