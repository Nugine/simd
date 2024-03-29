//! SIMD-accelerated Unicode validation and transcoding
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
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::wildcard_imports
)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod ascii;
mod utf16;
mod utf32;

mod multiversion;

pub use outref::{AsOut, Out};

// ------------------------------------------------------------------------------------------------

use vsimd::tools::{slice_mut, slice_parts};

/// Checks if `data` is a valid ASCII string.
#[inline]
#[must_use]
pub fn is_ascii(data: &[u8]) -> bool {
    let (src, len) = slice_parts(data);
    unsafe { crate::multiversion::is_ascii::auto(src, len) }
}

/// Converts ascii bytes to a string slice.
#[inline]
#[must_use]
pub fn from_ascii(data: &[u8]) -> Option<&str> {
    is_ascii(data).then(|| unsafe { core::str::from_utf8_unchecked(data) })
}

/// TODO: test, bench
#[inline]
#[must_use]
pub fn is_utf32le(data: &[u32]) -> bool {
    let (src, len) = slice_parts(data);
    unsafe { crate::multiversion::is_utf32le::auto(src, len) }
}

/// TODO: test, bench
#[inline]
pub fn utf32_swap_endianness_inplace(data: &mut [u32]) {
    let len = data.len();
    let dst = data.as_mut_ptr();
    let src = dst;
    unsafe { crate::multiversion::utf32_swap_endianness::auto(src, len, dst) }
}

/// TODO: test, bench
///
/// # Panics
/// This function asserts that `src.len() <= dst.len()`
#[inline]
#[must_use]
pub fn utf32_swap_endianness<'d>(src: &[u32], mut dst: Out<'d, [u32]>) -> &'d mut [u32] {
    assert!(src.len() <= dst.len());
    let len = src.len();
    let src = src.as_ptr();
    let dst = dst.as_mut_ptr();
    unsafe { crate::multiversion::utf32_swap_endianness::auto(src, len, dst) };
    unsafe { slice_mut(dst, len) }
}

/// TODO: test, bench
#[inline]
pub fn utf16_swap_endianness_inplace(data: &mut [u16]) {
    let len = data.len();
    let dst = data.as_mut_ptr();
    let src = dst;
    unsafe { crate::multiversion::utf16_swap_endianness::auto(src, len, dst) }
}

/// TODO: test, bench
///
/// # Panics
/// This function asserts that `src.len() <= dst.len()`
#[inline]
#[must_use]
pub fn utf16_swap_endianness<'d>(src: &[u16], mut dst: Out<'d, [u16]>) -> &'d mut [u16] {
    assert!(src.len() <= dst.len());
    let len = src.len();
    let src = src.as_ptr();
    let dst = dst.as_mut_ptr();
    unsafe { crate::multiversion::utf16_swap_endianness::auto(src, len, dst) };
    unsafe { slice_mut(dst, len) }
}
