use crate::{AppendHexDecode, AppendHexEncode, AsciiCase, Error, FromHexDecode, FromHexEncode};

use vsimd::tools::{alloc_uninit_bytes, assume_init, slice_mut, slice_parts};

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

fn decode_to_boxed_bytes(data: &[u8]) -> Result<Box<[u8]>, Error> {
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

fn decode_append_vec(src: &[u8], buf: &mut Vec<u8>) -> Result<(), Error> {
    if src.is_empty() {
        return Ok(());
    }

    ensure!(src.len() % 2 == 0);
    let m = src.len() / 2;

    buf.reserve_exact(m);
    let prev_len = buf.len();

    unsafe {
        let (src, len) = slice_parts(src);
        let dst = buf.as_mut_ptr().add(prev_len);
        crate::multiversion::decode::auto(src, len, dst)?;

        buf.set_len(prev_len + m);
        Ok(())
    }
}

fn encode_to_boxed_str(data: &[u8], case: AsciiCase) -> Box<str> {
    if data.is_empty() {
        return Box::from("");
    }

    unsafe {
        let m = data.len() * 2;
        assert!(m <= usize::MAX / 2);

        let mut uninit_buf = alloc_uninit_bytes(m);

        let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
        crate::multiversion::encode::auto(data, dst, case);

        let ptr = Box::into_raw(uninit_buf).cast::<u8>();
        Box::from_raw(core::str::from_utf8_unchecked_mut(slice_mut(ptr, m)))
    }
}

fn encode_append_vec(src: &[u8], buf: &mut Vec<u8>, case: AsciiCase) {
    if src.is_empty() {
        return;
    }

    unsafe {
        let m = src.len() * 2;
        assert!(m <= usize::MAX / 2);

        buf.reserve_exact(m);
        let prev_len = buf.len();

        let dst = buf.as_mut_ptr().add(prev_len);
        crate::multiversion::encode::auto(src, dst, case);

        buf.set_len(prev_len + m);
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromHexDecode for Box<[u8]> {
    #[inline]
    fn from_hex_decode(data: &[u8]) -> Result<Self, Error> {
        decode_to_boxed_bytes(data)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromHexDecode for Vec<u8> {
    #[inline]
    fn from_hex_decode(data: &[u8]) -> Result<Self, Error> {
        let ans = decode_to_boxed_bytes(data)?;
        Ok(Vec::from(ans))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromHexEncode for Box<[u8]> {
    #[inline]
    #[must_use]
    fn from_hex_encode(data: &[u8], case: AsciiCase) -> Self {
        let ans = encode_to_boxed_str(data, case);
        ans.into_boxed_bytes()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromHexEncode for Box<str> {
    #[inline]
    #[must_use]
    fn from_hex_encode(data: &[u8], case: AsciiCase) -> Self {
        encode_to_boxed_str(data, case)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromHexEncode for Vec<u8> {
    #[inline]
    #[must_use]
    fn from_hex_encode(data: &[u8], case: AsciiCase) -> Self {
        let ans = encode_to_boxed_str(data, case);
        Vec::from(ans.into_boxed_bytes())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromHexEncode for String {
    #[inline]
    #[must_use]
    fn from_hex_encode(data: &[u8], case: AsciiCase) -> Self {
        let ans = encode_to_boxed_str(data, case);
        String::from(ans)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl AppendHexEncode for Vec<u8> {
    #[inline]
    fn append_hex_encode(src: &[u8], dst: &mut Self, case: AsciiCase) {
        encode_append_vec(src, dst, case);
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl AppendHexEncode for String {
    #[inline]
    fn append_hex_encode(src: &[u8], dst: &mut Self, case: AsciiCase) {
        unsafe { encode_append_vec(src, dst.as_mut_vec(), case) }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl AppendHexDecode for Vec<u8> {
    #[inline]
    fn append_hex_decode(src: &[u8], dst: &mut Self) -> Result<(), Error> {
        decode_append_vec(src, dst)
    }
}
