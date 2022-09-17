use crate::{AsciiCase, Error, FromHexDecode, FromHexEncode};

use vsimd::tools::{alloc_uninit_bytes, assume_init, slice_mut};

use alloc::boxed::Box;

impl FromHexDecode for Box<[u8]> {
    #[inline]
    fn from_hex_decode(data: &[u8]) -> Result<Self, Error> {
        if data.is_empty() {
            return Ok(Box::from([]));
        }

        ensure!(data.len() % 2 == 0);

        unsafe {
            let mut uninit_buf = alloc_uninit_bytes(data.len() / 2);

            let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
            let src = data.as_ptr();
            let len = data.len();
            crate::multiversion::decode::auto(src, len, dst)?;

            Ok(assume_init(uninit_buf))
        }
    }
}

impl FromHexDecode for Vec<u8> {
    #[inline]
    fn from_hex_decode(data: &[u8]) -> Result<Self, Error> {
        let ans = <Box<[u8]> as FromHexDecode>::from_hex_decode(data)?;
        Ok(Vec::from(ans))
    }
}

impl FromHexEncode for Box<[u8]> {
    #[inline]
    #[must_use]
    fn from_hex_encode(data: &[u8], case: AsciiCase) -> Self {
        if data.is_empty() {
            return Box::from([]);
        }

        unsafe {
            assert!(data.len() <= usize::MAX / 4);

            let mut uninit_buf = alloc_uninit_bytes(data.len() * 2);

            let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
            crate::multiversion::encode::auto(data, dst, case);

            let len = uninit_buf.len();
            let ptr = Box::into_raw(uninit_buf).cast::<u8>();
            Box::from_raw(slice_mut(ptr, len))
        }
    }
}

impl FromHexEncode for Box<str> {
    #[inline]
    #[must_use]
    fn from_hex_encode(data: &[u8], case: AsciiCase) -> Self {
        let ans = <Box<[u8]> as FromHexEncode>::from_hex_encode(data, case);

        unsafe {
            let len = ans.len();
            let ptr = Box::into_raw(ans).cast();
            Box::from_raw(core::str::from_utf8_unchecked_mut(slice_mut(ptr, len)))
        }
    }
}

impl FromHexEncode for Vec<u8> {
    #[inline]
    #[must_use]
    fn from_hex_encode(data: &[u8], case: AsciiCase) -> Self {
        let ans = <Box<[u8]> as FromHexEncode>::from_hex_encode(data, case);
        Vec::from(ans)
    }
}

impl FromHexEncode for String {
    #[inline]
    #[must_use]
    fn from_hex_encode(data: &[u8], case: AsciiCase) -> Self {
        let ans = <Box<str> as FromHexEncode>::from_hex_encode(data, case);
        String::from(ans)
    }
}
