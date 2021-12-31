use crate::auto::*;
use crate::{AsciiCase, Error, OutBuf, ERROR};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
use simd_abstraction::tools::alloc_uninit_bytes;

/// Encodes `src` and returns [`Box<str>`].
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_to_boxed_str(src: &[u8], case: AsciiCase) -> Box<str> {
    use core::slice;
    use core::str;

    if src.is_empty() {
        return Box::from("");
    }

    unsafe {
        // src.len() <= isize::MAX, so (src.len() * 2) never overflows
        let mut uninit_buf = alloc_uninit_bytes(src.len() * 2);
        encode(src, OutBuf::from_uninit_mut(&mut *uninit_buf), case).unwrap();

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
        decode(src, OutBuf::from_uninit_mut(&mut *uninit_buf))?;

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
/// + The content of `src` is invalid.
///
#[inline]
pub fn encode_as_str<'s, 'd>(
    src: &'s [u8],
    dst: OutBuf<'d, u8>,
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
        let dst = OutBuf::from_uninit_mut(&mut dst);
        let case = AsciiCase::Lower;
        encode_as_str(src, dst, case).unwrap()
    };
    assert_eq!(ans, "68656c6c6f");
}
