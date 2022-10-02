use crate::fallback::decoded_length;
use crate::fallback::encoded_length_unchecked;
use crate::{Base64, FromBase64Decode, FromBase64Encode};

use vsimd::tools::{alloc_uninit_bytes, assume_init, slice_mut};

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase64Decode for Box<[u8]> {
    #[inline]
    fn from_base64_decode(base64: &Base64, data: &[u8]) -> Result<Self, crate::Error> {
        if data.is_empty() {
            return Ok(Box::from([]));
        }

        unsafe {
            let (n, m) = decoded_length(data, base64.config)?;

            // safety: 0 < m < isize::MAX
            let mut uninit_buf = alloc_uninit_bytes(m);

            let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
            let src: *const u8 = data.as_ptr();
            crate::multiversion::decode::auto(src, dst, n, base64.config)?;

            Ok(assume_init(uninit_buf))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase64Decode for Vec<u8> {
    #[inline]
    fn from_base64_decode(base64: &Base64, data: &[u8]) -> Result<Self, crate::Error> {
        let ans = <Box<[u8]> as FromBase64Decode>::from_base64_decode(base64, data)?;
        Ok(Vec::from(ans))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase64Encode for Box<[u8]> {
    #[inline]
    fn from_base64_encode(base64: &Base64, data: &[u8]) -> Self {
        if data.is_empty() {
            return Box::from([]);
        }

        unsafe {
            let m = encoded_length_unchecked(data.len(), base64.config);
            assert!(m <= usize::MAX / 2);

            let mut uninit_buf = alloc_uninit_bytes(m);

            let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
            crate::multiversion::encode::auto(data, dst, base64.config);

            let len = uninit_buf.len();
            let ptr = Box::into_raw(uninit_buf).cast::<u8>();
            Box::from_raw(slice_mut(ptr, len))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase64Encode for Box<str> {
    #[inline]
    fn from_base64_encode(base64: &Base64, data: &[u8]) -> Self {
        let ans = <Box<[u8]> as FromBase64Encode>::from_base64_encode(base64, data);

        unsafe {
            let len = ans.len();
            let ptr = Box::into_raw(ans).cast();
            Box::from_raw(core::str::from_utf8_unchecked_mut(slice_mut(ptr, len)))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase64Encode for Vec<u8> {
    #[inline]
    fn from_base64_encode(base64: &Base64, data: &[u8]) -> Self {
        let ans = <Box<[u8]> as FromBase64Encode>::from_base64_encode(base64, data);
        Vec::from(ans)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase64Encode for String {
    #[inline]
    fn from_base64_encode(base64: &Base64, data: &[u8]) -> Self {
        let ans = <Box<str> as FromBase64Encode>::from_base64_encode(base64, data);
        String::from(ans)
    }
}
