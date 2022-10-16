use crate::decode::decoded_length;
use crate::encode::encoded_length_unchecked;
use crate::{Base32, FromBase32Decode, FromBase32Encode};

use vsimd::tools::{alloc_uninit_bytes, assume_init, slice_mut};

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Decode for Box<[u8]> {
    #[inline]
    fn from_base32_decode(base32: &Base32, data: &[u8]) -> Result<Self, crate::Error> {
        if data.is_empty() {
            return Ok(Box::from([]));
        }

        unsafe {
            let (n, m) = decoded_length(data, base32.padding)?;

            // safety: 0 < m < isize::MAX
            let mut uninit_buf = alloc_uninit_bytes(m);

            let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
            let src: *const u8 = data.as_ptr();
            crate::multiversion::decode::auto(src, n, dst, base32.kind)?;

            Ok(assume_init(uninit_buf))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Decode for Vec<u8> {
    #[inline]
    fn from_base32_decode(base32: &Base32, data: &[u8]) -> Result<Self, crate::Error> {
        let ans = <Box<[u8]> as FromBase32Decode>::from_base32_decode(base32, data)?;
        Ok(Vec::from(ans))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Encode for Box<[u8]> {
    #[inline]
    fn from_base32_encode(base32: &Base32, data: &[u8]) -> Self {
        if data.is_empty() {
            return Box::from([]);
        }

        unsafe {
            let m = encoded_length_unchecked(data.len(), base32.padding);
            assert!(m <= usize::MAX / 2);

            let mut uninit_buf = alloc_uninit_bytes(m);

            let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
            crate::multiversion::encode::auto(data, dst, base32.kind, base32.padding);

            let len = uninit_buf.len();
            let ptr = Box::into_raw(uninit_buf).cast::<u8>();
            Box::from_raw(slice_mut(ptr, len))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Encode for Box<str> {
    #[inline]
    fn from_base32_encode(base32: &Base32, data: &[u8]) -> Self {
        let ans = <Box<[u8]> as FromBase32Encode>::from_base32_encode(base32, data);

        unsafe {
            let len = ans.len();
            let ptr = Box::into_raw(ans).cast();
            Box::from_raw(core::str::from_utf8_unchecked_mut(slice_mut(ptr, len)))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Encode for Vec<u8> {
    #[inline]
    fn from_base32_encode(base32: &Base32, data: &[u8]) -> Self {
        let ans = <Box<[u8]> as FromBase32Encode>::from_base32_encode(base32, data);
        Vec::from(ans)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Encode for String {
    #[inline]
    fn from_base32_encode(base32: &Base32, data: &[u8]) -> Self {
        let ans = <Box<str> as FromBase32Encode>::from_base32_encode(base32, data);
        String::from(ans)
    }
}
