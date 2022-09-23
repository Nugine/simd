//! SIMD-accelerated base32 encoding and decoding.
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! let bytes = b"hello world";
//! let base32 = base32_simd::BASE32;
//!
//! let encoded = base32.encode_type::<String>(bytes);
//! assert_eq!(&*encoded, "NBSWY3DPEB3W64TMMQ======");
//!
//! let decoded = base32.decode_type::<Vec<u8>>(encoded.as_bytes()).unwrap();
//! assert_eq!(&*decoded, bytes);
//! # }
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
    clippy::missing_inline_in_public_items
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

mod multiversion;

#[cfg(feature = "alloc")]
mod heap;

#[cfg(test)]
mod tests;

pub use outref::OutRef;

// -----------------------------------------------------------------------------

use crate::decode::decoded_length;
use crate::encode::encoded_length_unchecked;

use vsimd::base32::Kind;
use vsimd::base32::{BASE32HEX_CHARSET, BASE32_CHARSET};
use vsimd::tools::slice_mut;

/// Base32 variant
#[derive(Debug)]
pub struct Base32 {
    kind: Kind,
    padding: bool,
}

/// Base32 charset with padding.
pub const BASE32: Base32 = Base32 {
    kind: Kind::Base32,
    padding: true,
};

/// Base32Hex charset with padding.
pub const BASE32HEX: Base32 = Base32 {
    kind: Kind::Base32Hex,
    padding: true,
};

/// Base32 charset withnot padding.
pub const BASE32_NO_PAD: Base32 = Base32 {
    kind: Kind::Base32,
    padding: false,
};

/// Base32Hex charset withnot padding.
pub const BASE32HEX_NO_PAD: Base32 = Base32 {
    kind: Kind::Base32Hex,
    padding: false,
};

impl Base32 {
    /// Returns the character set.
    #[inline]
    #[must_use]
    pub fn charset(&self) -> &'static [u8; 32] {
        match self.kind {
            Kind::Base32 => BASE32_CHARSET,
            Kind::Base32Hex => BASE32HEX_CHARSET,
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
        if n % 8 == 0 {
            n / 8 * 5
        } else {
            (n / 8 + 1) * 5
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

    /// Checks whether `data` is a base32 string.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    #[inline]
    pub fn check(&self, data: &[u8]) -> Result<(), Error> {
        let (n, _) = decoded_length(data, self.padding)?;
        let src = unsafe { data.get_unchecked(..n) };
        crate::multiversion::check::auto(src, self.kind)
    }

    /// Encodes bytes to a base32 string.
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

    /// Encodes bytes to a base32 string and returns [`&mut str`](str).
    ///
    /// # Panics
    /// This function will panic if the length of `dst` is not enough.
    #[inline]
    #[must_use]
    pub fn encode_as_str<'s, 'd>(&'_ self, src: &'s [u8], dst: OutRef<'d, [u8]>) -> &'d mut str {
        let ans = self.encode(src, dst);
        unsafe { core::str::from_utf8_unchecked_mut(ans) }
    }

    /// Decodes a base32 string to bytes.
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
            self::multiversion::decode::auto(src, n, dst, self.kind)?;

            Ok(slice_mut(dst, m))
        }
    }

    /// Decodes a base32 string to bytes and writes inplace.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    #[inline]
    pub fn decode_inplace<'d>(&'_ self, data: &'d mut [u8]) -> Result<&'d mut [u8], Error> {
        unsafe {
            let (n, m) = decoded_length(data, self.padding)?;

            let dst: *mut u8 = data.as_mut_ptr();
            let src: *const u8 = dst;
            crate::multiversion::decode::auto(src, n, dst, self.kind)?;

            Ok(slice_mut(dst, m))
        }
    }

    /// Encodes bytes to a base32 string and returns a specified type.
    #[inline]
    #[must_use]
    pub fn encode_type<T: FromBase32Encode>(&self, data: &[u8]) -> T {
        T::from_base32_encode(self, data)
    }

    /// Decodes a base32 string to bytes and returns a specified type.
    #[inline]
    pub fn decode_type<T: FromBase32Decode>(&self, data: &[u8]) -> Result<T, Error> {
        T::from_base32_decode(self, data)
    }
}

/// Types that can represent a base32 string.
pub trait FromBase32Encode: Sized {
    /// Encodes bytes to a base32 string and returns the self type.
    fn from_base32_encode(base32: &Base32, data: &[u8]) -> Self;
}

/// Types that can be decoded from a base32 string.
pub trait FromBase32Decode: Sized {
    /// Decodes a base32 string to bytes case-insensitively and returns the self type.
    fn from_base32_decode(base32: &Base32, data: &[u8]) -> Result<Self, Error>;
}
