//! SIMD-accelerated hex encoding and decoding.
//!
//! # Examples
//!
//! ```
//! use hex_simd::AsciiCase;
//!
//! let bytes = b"Hello world!";
//!
//! let encoded = hex_simd::encode_type::<String>(bytes, AsciiCase::Lower);
//! assert_eq!(&*encoded, "48656c6c6f20776f726c6421");
//!
//! let decoded = hex_simd::decode_type::<Vec<u8>>(encoded.as_bytes()).unwrap();
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

mod multiversion;

#[cfg(feature = "alloc")]
mod heap;

#[cfg(test)]
mod tests;

pub use outref::OutRef;
pub use vsimd::ascii::AsciiCase;

// -------------------------------------------------------------------------------------------------

use vsimd::tools::slice_mut;

/// Calculates the encoded length.
///
/// # Panics
/// This function will panic if `n > isize::MAX`.
#[inline]
#[must_use]
pub const fn encoded_length(n: usize) -> usize {
    assert!(n <= usize::MAX / 2);
    n * 2
}

/// Calculates the decoded length.
#[inline]
pub fn decoded_length(n: usize) -> Result<usize, Error> {
    ensure!(n % 2 == 0);
    Ok(n / 2)
}

/// Checks whether `data` is a hex string.
///
/// # Errors
/// This function returns `Err` if the content of `data` is invalid.
#[inline]
pub fn check(data: &[u8]) -> Result<(), Error> {
    crate::multiversion::check::auto(data)
}

/// Encodes bytes to a hex string.
///
/// `case` specifies the ascii case of output.
///
/// # Panics
/// This function will panic if the length of `dst` is not enough.
#[inline]
#[must_use]
pub fn encode<'s, 'd>(src: &'s [u8], mut dst: OutRef<'d, [u8]>, case: AsciiCase) -> &'d mut [u8] {
    assert!(dst.len() / 2 >= src.len());
    let dst = dst.as_mut_ptr();
    unsafe {
        crate::multiversion::encode::auto(src, dst, case);
        slice_mut(dst, src.len() * 2)
    }
}

/// Decodes a hex string to bytes case-insensitively.
///
/// # Errors
/// This function returns `Err` if the content of `src` is invalid.
///
/// # Panics
/// This function will panic if the length of `dst` is not enough.
#[inline]
pub fn decode<'s, 'd>(src: &'s [u8], mut dst: OutRef<'d, [u8]>) -> Result<&'d mut [u8], Error> {
    ensure!(src.len() % 2 == 0);
    assert!(dst.len() >= src.len() / 2);

    let len = src.len();
    let dst = dst.as_mut_ptr();
    let src = src.as_ptr();
    unsafe {
        crate::multiversion::decode::auto(src, len, dst)?;
        Ok(slice_mut(dst, len / 2))
    }
}

/// Decodes a hex string to bytes case-insensitively and writes inplace.
///
/// # Errors
/// This function returns `Err` if the content of `data` is invalid.
#[inline]
pub fn decode_inplace(data: &mut [u8]) -> Result<&mut [u8], Error> {
    ensure!(data.len() % 2 == 0);
    unsafe {
        let len = data.len();
        let dst: *mut u8 = data.as_mut_ptr();
        let src: *const u8 = dst;
        crate::multiversion::decode::auto(src, len, dst)?;
        Ok(slice_mut(dst, len / 2))
    }
}

/// Encodes bytes to a hex string and returns [`&mut str`](str).
///
/// `case` specifies the ascii case of output.
///
/// # Panics
/// This function will panic if the length of `dst` is not enough.
#[inline]
#[must_use]
pub fn encode_as_str<'s, 'd>(src: &'s [u8], dst: OutRef<'d, [u8]>, case: AsciiCase) -> &'d mut str {
    let ans = encode(src, dst, case);
    unsafe { core::str::from_utf8_unchecked_mut(ans) }
}

/// Types that can be decoded from a hex string.
pub trait FromHexDecode: Sized {
    /// Decodes a hex string to bytes case-insensitively and returns the self type.
    fn from_hex_decode(data: &[u8]) -> Result<Self, Error>;
}

/// Types that can represent a hex string.
pub trait FromHexEncode: Sized {
    /// Encodes bytes to a hex string and returns the self type.
    fn from_hex_encode(data: &[u8], case: AsciiCase) -> Self;
}

/// Encodes bytes to a hex string and returns a specified type.
#[inline]
#[must_use]
pub fn encode_type<T: FromHexEncode>(data: &[u8], case: AsciiCase) -> T {
    T::from_hex_encode(data, case)
}

/// Decodes a hex string to bytes case-insensitively and returns a specified type.
#[inline]
pub fn decode_type<T: FromHexDecode>(data: &[u8]) -> Result<T, Error> {
    T::from_hex_decode(data)
}
