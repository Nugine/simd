//! TODO

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

mod check;
mod decode;
mod encode;

pub mod multiversion;

pub use simd_abstraction::ascii::AsciiCase;
use simd_abstraction::item_group;
pub use simd_abstraction::tools::OutBuf;

// -------------------------------------------------------------------------------------------------

use self::error::ERROR;
use simd_abstraction::tools::slice_mut;

/// Checks whether `data` is a hex string.
#[inline]
pub fn check(data: &[u8]) -> bool {
    crate::multiversion::check::auto_indirect(data)
}

/// Encodes `src` with a given ascii case and writes to `dst`.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `dst` is not enough.
#[inline]
pub fn encode<'s, 'd>(
    src: &'s [u8],
    mut dst: OutBuf<'d>,
    case: AsciiCase,
) -> Result<&'d mut [u8], Error> {
    if dst.len() / 2 < src.len() {
        return Err(ERROR);
    }
    unsafe {
        let dst = dst.as_mut_ptr();
        crate::multiversion::encode_raw::auto_indirect(src, dst, case);
        Ok(slice_mut(dst, src.len() * 2))
    }
}

/// Decodes `src` case-insensitively and writes to `dst`.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `dst` is not enough.
/// + The content of `src` is invalid.
#[inline]
pub fn decode<'s, 'd>(src: &'s [u8], mut dst: OutBuf<'d>) -> Result<&'d mut [u8], Error> {
    let len = src.len();
    if len % 2 != 0 || dst.len() < len / 2 {
        return Err(ERROR);
    }
    unsafe {
        let dst: *mut u8 = dst.as_mut_ptr();
        let src: *const u8 = src.as_ptr();
        crate::multiversion::decode_raw::auto_indirect(src, len, dst)?;
        Ok(slice_mut(dst, len / 2))
    }
}

/// Decodes `data` case-insensitively and writes inplace.
///
/// # Errors
/// This function returns `Err` if:
///
/// + The content of `data` is invalid.
#[inline]
pub fn decode_inplace(data: &mut [u8]) -> Result<&mut [u8], Error> {
    let len = data.len();
    if len % 2 != 0 {
        return Err(ERROR);
    }
    unsafe {
        let dst: *mut u8 = data.as_mut_ptr();
        let src: *const u8 = dst;
        crate::multiversion::decode_raw::auto_indirect(src, len, dst)?;
        Ok(slice_mut(dst, len / 2))
    }
}

/// Encodes `src` to `dst` and returns [`&mut str`](str).
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `dst` is not enough.
#[inline]
pub fn encode_as_str<'s, 'd>(
    src: &'s [u8],
    dst: OutBuf<'d>,
    case: AsciiCase,
) -> Result<&'d mut str, Error> {
    let ans = encode(src, dst, case)?;
    Ok(unsafe { core::str::from_utf8_unchecked_mut(ans) })
}

#[cfg(feature = "alloc")]
item_group! {
    use alloc::boxed::Box;
    use simd_abstraction::tools::alloc_uninit_bytes;
}

/// Encodes `data` and returns [`Box<str>`].
///
/// # Panics
/// This function panics if:
///
/// + The encoded length of `data` is greater than `isize::MAX`.
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_to_boxed_str(data: &[u8], case: AsciiCase) -> Box<str> {
    if data.is_empty() {
        return Box::from("");
    }

    unsafe {
        assert!(data.len() <= usize::MAX / 4);

        let mut uninit_buf = alloc_uninit_bytes(data.len() * 2);

        let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
        crate::multiversion::encode_raw::auto_indirect(data, dst, case);

        let len = uninit_buf.len();
        let ptr = Box::into_raw(uninit_buf).cast::<u8>();
        Box::from_raw(core::str::from_utf8_unchecked_mut(slice_mut(ptr, len)))
    }
}

/// Decodes `data` and returns [`Box<[u8]>`](Box).
///
/// # Errors
/// This function returns `Err` if:
///
/// + The content of `data` is invalid.
#[cfg(feature = "alloc")]
#[inline]
pub fn decode_to_boxed_bytes(data: &[u8]) -> Result<Box<[u8]>, Error> {
    if data.is_empty() {
        return Ok(Box::from([]));
    }

    unsafe {
        if data.len() % 2 != 0 {
            return Err(ERROR);
        }

        let mut uninit_buf = alloc_uninit_bytes(data.len() / 2);

        let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
        let src = data.as_ptr();
        let len = data.len();
        crate::multiversion::decode_raw::auto_indirect(src, len, dst)?;

        let len = uninit_buf.len();
        let ptr = Box::into_raw(uninit_buf).cast::<u8>();
        Ok(Box::from_raw(core::ptr::slice_from_raw_parts_mut(ptr, len)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str() {
        use core::mem::MaybeUninit;
        let src = "hello";
        let mut dst = [MaybeUninit::uninit(); 10];
        let ans = {
            let src = src.as_bytes();
            let dst = OutBuf::uninit(&mut dst);
            let case = AsciiCase::Lower;
            encode_as_str(src, dst, case).unwrap()
        };
        assert_eq!(ans, "68656c6c6f");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_alloc() {
        let src = "hello".as_bytes();

        let ans = encode_to_boxed_str(src, AsciiCase::Lower);
        assert_eq!(&*ans, "68656c6c6f");

        let ans = decode_to_boxed_bytes(ans.as_bytes()).unwrap();
        assert_eq!(&*ans, src);
    }
}
