//! SIMD-accelerated Unicode validation and transcoding
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

mod utf32;

mod multiversion;

/// Checks if `data` is a valid ASCII string, in constant-time.
///
/// This function always scans the entire input
/// without data-dependent branches or lookup tables.
///
/// This function is faster than the short-circuiting version
/// if the inputs are mostly valid ASCII strings.
#[inline]
pub fn is_ascii_ct(data: &[u8]) -> bool {
    simd_abstraction::ascii::multiversion::is_ascii_ct::auto_indirect(data)
}

/// TODO: test, bench
#[inline]
pub fn is_utf32le_ct(data: &[u32]) -> bool {
    crate::multiversion::is_utf32le_ct::auto_indirect(data)
}
