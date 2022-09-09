//! SIMD-accelerated hex encoding and decoding.
//!
//! # Examples
//!
//! ```
//! use hex_simd::AsciiCase;
//!
//! let bytes = b"Hello world!";
//!
//! let encoded = hex_simd::encode_to_boxed_str(bytes, AsciiCase::Lower);
//! assert_eq!(&*encoded, "48656c6c6f20776f726c6421");
//!
//! let decoded = hex_simd::decode_to_boxed_bytes(encoded.as_bytes()).unwrap();
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

#[cfg(test)]
mod tests;

pub use outref::OutRef;
pub use vsimd::ascii::AsciiCase;

// -------------------------------------------------------------------------------------------------

use vsimd::item_group;
use vsimd::tools::slice_mut;

#[cfg(feature = "alloc")]
item_group! {
    use alloc::boxed::Box;
    use vsimd::tools::{alloc_uninit_bytes, assume_init};
}

/// Checks whether `data` is a hex string.
#[inline]
#[must_use]
pub fn check(data: &[u8]) -> bool {
    crate::multiversion::check::auto_indirect(data)
}

/// Encodes `src` with a given ascii case and writes to `dst`.
///
/// # Panics
/// This function will panic if the length of `dst` is not enough.
#[inline]
#[must_use]
pub fn encode<'s, 'd>(src: &'s [u8], mut dst: OutRef<'d, [u8]>, case: AsciiCase) -> &'d mut [u8] {
    assert!(dst.len() / 2 >= src.len());
    let dst = dst.as_mut_ptr();
    unsafe {
        crate::multiversion::encode::auto_indirect(src, dst, case);
        slice_mut(dst, src.len() * 2)
    }
}

/// Decodes `src` case-insensitively and writes to `dst`.
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
        crate::multiversion::decode::auto_indirect(src, len, dst)?;
        Ok(slice_mut(dst, len / 2))
    }
}

/// Decodes `data` case-insensitively and writes inplace.
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
        crate::multiversion::decode::auto_indirect(src, len, dst)?;
        Ok(slice_mut(dst, len / 2))
    }
}

/// Encodes `src` to `dst` and returns [`&mut str`](str).
///
/// # Panics
/// This function will panic if the length of `dst` is not enough.
#[inline]
#[must_use]
pub fn encode_as_str<'s, 'd>(src: &'s [u8], dst: OutRef<'d, [u8]>, case: AsciiCase) -> &'d mut str {
    let ans = encode(src, dst, case);
    unsafe { core::str::from_utf8_unchecked_mut(ans) }
}

/// Encodes `data` and returns [`Box<str>`].
///
/// # Panics
/// This function will panic if the encoded length of `data` is greater than `isize::MAX`.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
#[inline]
#[must_use]
pub fn encode_to_boxed_str(data: &[u8], case: AsciiCase) -> Box<str> {
    if data.is_empty() {
        return Box::from("");
    }

    unsafe {
        assert!(data.len() <= usize::MAX / 4);

        let mut uninit_buf = alloc_uninit_bytes(data.len() * 2);

        let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
        crate::multiversion::encode::auto_indirect(data, dst, case);

        let len = uninit_buf.len();
        let ptr = Box::into_raw(uninit_buf).cast::<u8>();
        Box::from_raw(core::str::from_utf8_unchecked_mut(slice_mut(ptr, len)))
    }
}

/// Decodes `data` and returns [`Box<[u8]>`](Box).
///
/// # Errors
/// This function returns `Err` if the content of `data` is invalid.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
#[inline]
pub fn decode_to_boxed_bytes(data: &[u8]) -> Result<Box<[u8]>, Error> {
    if data.is_empty() {
        return Ok(Box::from([]));
    }

    ensure!(data.len() % 2 == 0);

    unsafe {
        let mut uninit_buf = alloc_uninit_bytes(data.len() / 2);

        let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
        let src = data.as_ptr();
        let len = data.len();
        crate::multiversion::decode::auto_indirect(src, len, dst)?;

        Ok(assume_init(uninit_buf))
    }
}
