//! TODO

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
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

mod error;
pub use self::error::Error;

mod spec;

mod format;
mod parse;

mod multiversion;

#[cfg(test)]
mod tests;

pub use simd_abstraction::ascii::AsciiCase;
pub use simd_abstraction::tools::OutRef;

// -------------------------------------------------------------------------------------------------

use crate::error::ERROR;

use simd_abstraction::tools::read;

/// Parses an UUID from arbitrary bytes.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `src` doesn't match any UUID format variants.
/// + The content of `src` is invalid.
#[inline]
pub fn parse<'s, 'd>(
    src: &'s [u8],
    mut dst: OutRef<'d, [u8; 16]>,
) -> Result<&'d mut [u8; 16], Error> {
    let n = src.len();

    if n == 32 {
        unsafe {
            let src = src.as_ptr();
            let dst = dst.as_mut_ptr().cast::<u8>();
            crate::multiversion::parse_simple_raw::auto_indirect(src, dst)?;
            return Ok(&mut *dst.cast());
        }
    }

    unsafe {
        let src = match n {
            36 => src.as_ptr(),
            // Microsoft GUID
            38 => {
                let src = src.as_ptr();
                if read(src, 0) != b'{' || read(src, 37) != b'}' {
                    return Err(ERROR);
                }
                src.add(1)
            }
            // URN prefixed UUID
            45 => src.strip_prefix(b"urn:uuid:").ok_or(ERROR)?.as_ptr(),
            _ => return Err(ERROR),
        };
        let dst = dst.as_mut_ptr().cast::<u8>();
        crate::multiversion::parse_hyphenated_raw::auto_indirect(src, dst)?;
        Ok(&mut *dst.cast())
    }
}

/// Parses a simple UUID from arbitrary bytes.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `src` doesn't match the "simple" format.
/// + The content of `src` is invalid.
#[inline]
pub fn parse_simple<'s, 'd>(
    src: &'s [u8],
    mut dst: OutRef<'d, [u8; 16]>,
) -> Result<&'d mut [u8; 16], Error> {
    if src.len() != 32 {
        return Err(ERROR);
    }
    unsafe {
        let src = src.as_ptr();
        let dst = dst.as_mut_ptr().cast::<u8>();
        crate::multiversion::parse_simple_raw::auto_indirect(src, dst)?;
        Ok(&mut *dst.cast())
    }
}

/// Parses a hyphenated UUID from arbitrary bytes.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `src` doesn't match the "hyphenated" format.
/// + The content of `src` is invalid.
#[inline]
pub fn parse_hyphenated<'s, 'd>(
    src: &'s [u8],
    mut dst: OutRef<'d, [u8; 16]>,
) -> Result<&'d mut [u8; 16], Error> {
    if src.len() != 36 {
        return Err(ERROR);
    }
    unsafe {
        let src = src.as_ptr();
        let dst = dst.as_mut_ptr().cast::<u8>();
        crate::multiversion::parse_hyphenated_raw::auto_indirect(src, dst)?;
        Ok(&mut *dst.cast())
    }
}

/// Formats `src` to a simple UUID string.
#[inline]
pub fn format_simple<'s, 'd>(
    src: &'s [u8; 16],
    mut dst: OutRef<'d, [u8; 32]>,
    case: AsciiCase,
) -> &'d mut [u8; 32] {
    unsafe {
        let src = src.as_ptr();
        let dst = dst.as_mut_ptr().cast::<u8>();
        crate::multiversion::format_simple_raw::auto_indirect(src, dst, case);
        &mut *dst.cast()
    }
}

/// Formats `src` to a hyphenated UUID string.
#[inline]
pub fn format_hyphenated<'s, 'd>(
    src: &'s [u8; 16],
    mut dst: OutRef<'d, [u8; 36]>,
    case: AsciiCase,
) -> &'d mut [u8; 36] {
    unsafe {
        let src = src.as_ptr();
        let dst = dst.as_mut_ptr().cast::<u8>();
        crate::multiversion::format_hyphenated_raw::auto_indirect(src, dst, case);
        &mut *dst.cast()
    }
}
