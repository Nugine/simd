use core::mem::MaybeUninit;

use crate::auto::*;
use crate::{AsciiCase, Error, OutBuf, ERROR};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

pub use simd_abstraction::common::hex::Hex;

#[cfg(feature = "alloc")]
use simd_abstraction::tools::alloc_uninit_bytes;

/// Encodes `src` and returns [`Box<str>`].
///
/// # Panics
/// This function panics if:
///
/// + The encoded length of `src` is greater than `isize::MAX`.
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_to_boxed_str(src: &[u8], case: AsciiCase) -> Box<str> {
    use core::slice;
    use core::str;

    if src.is_empty() {
        return Box::from("");
    }

    unsafe {
        assert!(src.len() * 2 <= (isize::MAX as usize));

        let mut uninit_buf = alloc_uninit_bytes(src.len() * 2);
        encode(src, OutBuf::uninit(&mut *uninit_buf), case).unwrap();

        let len = uninit_buf.len();
        let ptr = Box::into_raw(uninit_buf).cast::<u8>();
        let buf = slice::from_raw_parts_mut(ptr, len);
        Box::from_raw(str::from_utf8_unchecked_mut(buf))
    }
}

/// Decodes `src` and returns [`Box<[u8]>`](Box).
///
/// # Errors
/// This function returns `Err` if:
///
/// + The content of `src` is invalid.
///
#[cfg(feature = "alloc")]
#[inline]
pub fn decode_to_boxed_bytes(src: &[u8]) -> Result<Box<[u8]>, Error> {
    use core::slice;

    if src.is_empty() {
        return Ok(Box::from([]));
    }

    unsafe {
        if src.len() % 2 != 0 {
            return Err(ERROR);
        }
        let mut uninit_buf = alloc_uninit_bytes(src.len() / 2);
        decode(src, OutBuf::uninit(&mut *uninit_buf))?;

        let len = uninit_buf.len();
        let ptr = Box::into_raw(uninit_buf).cast::<u8>();
        let buf = slice::from_raw_parts_mut(ptr, len);
        Ok(Box::from_raw(buf))
    }
}

/// Encodes `src` to `dst` and returns [`&mut str`](str).
///
/// # Errors
/// This function returns `Err` if:
///
/// + The length of `dst` is not enough.
///
#[inline]
pub fn encode_as_str<'s, 'd>(
    src: &'s [u8],
    dst: OutBuf<'d>,
    case: AsciiCase,
) -> Result<&'d mut str, Error> {
    let ans = encode(src, dst, case)?;
    Ok(unsafe { core::str::from_utf8_unchecked_mut(ans) })
}

#[test]
fn test_alloc() {
    let src = "hello".as_bytes();

    let ans = encode_to_boxed_str(src, AsciiCase::Lower);
    assert_eq!(&*ans, "68656c6c6f");

    let ans = decode_to_boxed_bytes(ans.as_bytes()).unwrap();
    assert_eq!(&*ans, src);
}

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

/// Encodes `src` to a hex string in network byte order
#[inline]
pub fn encode_u64(src: u64, case: AsciiCase) -> Hex<16> {
    unsafe {
        let mut this: MaybeUninit<Hex<16>> = MaybeUninit::uninit();
        let src = &src.to_be_bytes();
        let dst: *mut u8 = this.as_mut_ptr().cast();
        let table = match case {
            AsciiCase::Lower => crate::fallback::FULL_LOWER_TABLE,
            AsciiCase::Upper => crate::fallback::FULL_UPPER_TABLE,
        };
        crate::fallback::encode_unchecked(src, dst, table);
        this.assume_init()
    }
}

#[test]
fn test_hex_u64() {
    let src = 0x1234_5678_9abc_def0;

    let hex = encode_u64(src, AsciiCase::Lower);
    assert_eq!(hex.as_str(), format!("{:x}", src));

    let hex = encode_u64(src, AsciiCase::Upper);
    assert_eq!(hex.as_str(), format!("{:X}", src));
}
