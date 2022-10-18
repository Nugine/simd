//! SIMD-accelerated base64 encoding and decoding.
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! let bytes = b"hello world";
//! let base64 = base64_simd::STANDARD;
//!
//! let encoded = base64.encode_type::<String>(bytes);
//! assert_eq!(&*encoded, "aGVsbG8gd29ybGQ=");
//!
//! let decoded = base64.decode_type::<Vec<u8>>(encoded.as_bytes()).unwrap();
//! assert_eq!(&*decoded, bytes);
//! # }
//! ```
//!
#![doc=vsimd::shared_docs!()]
//
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]
//
#![deny(
    missing_debug_implementations,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
    clippy::missing_inline_in_public_items
)]
#![warn(clippy::todo)]
#![allow(
    clippy::inline_always,
    clippy::wildcard_imports,
    clippy::module_name_repetitions,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::items_after_statements,
    clippy::match_same_arms,
    clippy::verbose_bit_mask
)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod error;
pub use self::error::Error;

mod alsw;
mod ascii;
mod check;
mod decode;
mod encode;

mod multiversion;

#[cfg(feature = "alloc")]
mod heap;

mod forgiving;
pub use self::forgiving::*;

#[cfg(test)]
mod tests;

pub use outref::OutRef;

// -----------------------------------------------------------------------------

use crate::decode::decoded_length;
use crate::encode::encoded_length_unchecked;

use vsimd::tools::slice_mut;

const STANDARD_CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL_SAFE_CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Base64 variant
#[derive(Debug)]
pub struct Base64 {
    config: Config,
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Standard,
    UrlSafe,
}

#[derive(Debug, Clone, Copy)]
struct Config {
    kind: Kind,
    extra: Extra,
}

#[derive(Debug, Clone, Copy)]
enum Extra {
    Pad,
    NoPad,
    Forgiving,
}

impl Extra {
    /// Whether to add padding when encoding
    #[inline(always)]
    #[must_use]
    const fn padding(self) -> bool {
        match self {
            Extra::Pad => true,
            Extra::NoPad => false,
            Extra::Forgiving => true,
        }
    }

    #[inline(always)]
    #[must_use]
    const fn forgiving(self) -> bool {
        match self {
            Extra::Pad => false,
            Extra::NoPad => false,
            Extra::Forgiving => true,
        }
    }
}

/// Standard charset with padding.
pub const STANDARD: Base64 = Base64 {
    config: Config {
        kind: Kind::Standard,
        extra: Extra::Pad,
    },
};

/// URL-Safe charset with padding.
pub const URL_SAFE: Base64 = Base64 {
    config: Config {
        kind: Kind::UrlSafe,
        extra: Extra::Pad,
    },
};

/// Standard charset without padding.
pub const STANDARD_NO_PAD: Base64 = Base64 {
    config: Config {
        kind: Kind::Standard,
        extra: Extra::NoPad,
    },
};

/// URL-Safe charset without padding.
pub const URL_SAFE_NO_PAD: Base64 = Base64 {
    config: Config {
        kind: Kind::UrlSafe,
        extra: Extra::NoPad,
    },
};

/// Standard charset with forgiving behavior.
///
/// This configuration
/// + adds padding characters when encoding.
/// + allows non-zero trailing bits when decoding.
pub const STANDARD_FORGIVING: Base64 = Base64 {
    config: Config {
        kind: Kind::Standard,
        extra: Extra::Forgiving,
    },
};

/// URL-Safe charset with forgiving behavior.
///
/// This configuration
/// + adds padding characters when encoding.
/// + allows non-zero trailing bits when decoding.
///
pub const URL_SAFE_FORGIVING: Base64 = Base64 {
    config: Config {
        kind: Kind::UrlSafe,
        extra: Extra::Forgiving,
    },
};

impl Base64 {
    /// Returns the character set.
    #[inline]
    #[must_use]
    pub const fn charset(&self) -> &[u8; 64] {
        match self.config.kind {
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
        encoded_length_unchecked(n, self.config)
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
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is partially invalid.
    #[inline]
    pub fn decoded_length(&self, data: &[u8]) -> Result<usize, Error> {
        let (_, m) = decoded_length(data, self.config)?;
        Ok(m)
    }

    /// Checks whether `data` is a base64 string.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    #[inline]
    pub fn check(&self, data: &[u8]) -> Result<(), Error> {
        let (n, _) = decoded_length(data, self.config)?;
        let src = unsafe { data.get_unchecked(..n) };
        crate::multiversion::check::auto(src, self.config)
    }

    /// Encodes bytes to a base64 string.
    ///
    /// # Panics
    /// This function will panic if the length of `dst` is not enough.
    #[inline]
    #[must_use]
    pub fn encode<'s, 'd>(&'_ self, src: &'s [u8], mut dst: OutRef<'d, [u8]>) -> &'d mut [u8] {
        unsafe {
            let m = encoded_length_unchecked(src.len(), self.config);
            assert!(dst.len() >= m);

            let dst = dst.as_mut_ptr();
            self::multiversion::encode::auto(src, dst, self.config);

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
            let (n, m) = decoded_length(src, self.config)?;
            assert!(dst.len() >= m);

            let src = src.as_ptr();
            let dst = dst.as_mut_ptr();
            self::multiversion::decode::auto(src, dst, n, self.config)?;

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
            let (n, m) = decoded_length(data, self.config)?;

            let dst: *mut u8 = data.as_mut_ptr();
            let src: *const u8 = dst;
            self::multiversion::decode::auto(src, dst, n, self.config)?;

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
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    #[inline]
    pub fn decode_type<T: FromBase64Decode>(&self, data: &[u8]) -> Result<T, Error> {
        T::from_base64_decode(self, data)
    }

    /// Encodes bytes to a base64 string and appends to a specified type.
    #[inline]
    pub fn encode_append<T: AppendBase64Encode>(&self, src: &[u8], dst: &mut T) {
        T::append_base64_encode(self, src, dst);
    }

    /// Decodes a base64 string to bytes and appends to a specified type.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `src` is invalid.
    #[inline]
    pub fn decode_append<T: AppendBase64Decode>(&self, src: &[u8], dst: &mut T) -> Result<(), Error> {
        T::append_base64_decode(self, src, dst)
    }
}

/// Types that can represent a base64 string.
pub trait FromBase64Encode: Sized {
    /// Encodes bytes to a base64 string and returns the self type.
    fn from_base64_encode(base64: &Base64, data: &[u8]) -> Self;
}

/// Types that can be decoded from a base64 string.
pub trait FromBase64Decode: Sized {
    /// Decodes a base64 string to bytes and returns the self type.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    fn from_base64_decode(base64: &Base64, data: &[u8]) -> Result<Self, Error>;
}

/// Types that can append a base64 string.
pub trait AppendBase64Encode: FromBase64Encode {
    /// Encodes bytes to a base64 string and appends into the self type.
    fn append_base64_encode(base64: &Base64, src: &[u8], dst: &mut Self);
}

/// Types that can append bytes decoded from from a base64 string.
pub trait AppendBase64Decode: FromBase64Decode {
    /// Decodes a base64 string to bytes and appends to the self type.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `src` is invalid.
    fn append_base64_decode(base64: &Base64, src: &[u8], dst: &mut Self) -> Result<(), Error>;
}
