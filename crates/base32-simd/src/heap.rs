use crate::decode::decoded_length;
use crate::encode::encoded_length_unchecked;
use crate::{AppendBase32Decode, AppendBase32Encode, Base32, Error, FromBase32Decode, FromBase32Encode};

use vsimd::tools::{alloc_uninit_bytes, assume_init, slice_mut};

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

fn decode_to_boxed_bytes(base32: &Base32, src: &[u8]) -> Result<Box<[u8]>, Error> {
    if src.is_empty() {
        return Ok(Box::from([]));
    }

    unsafe {
        let (n, m) = decoded_length(src, base32.padding)?;

        // safety: 0 < m < isize::MAX
        let mut uninit_buf = alloc_uninit_bytes(m);

        let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
        let src: *const u8 = src.as_ptr();
        crate::multiversion::decode::auto(src, n, dst, base32.kind)?;

        Ok(assume_init(uninit_buf))
    }
}

fn decode_append_vec(base32: &Base32, src: &[u8], buf: &mut Vec<u8>) -> Result<(), Error> {
    if src.is_empty() {
        return Ok(());
    }

    let (n, m) = decoded_length(src, base32.padding)?;

    buf.reserve_exact(m);
    let prev_len = buf.len();

    unsafe {
        let dst: *mut u8 = buf.as_mut_ptr().add(prev_len);
        let src: *const u8 = src.as_ptr();
        crate::multiversion::decode::auto(src, n, dst, base32.kind)?;

        buf.set_len(prev_len + m);
        Ok(())
    }
}

fn encode_to_boxed_str(base32: &Base32, src: &[u8]) -> Box<str> {
    if src.is_empty() {
        return Box::from("");
    }

    unsafe {
        let m = encoded_length_unchecked(src.len(), base32.padding);
        assert!(m <= usize::MAX / 2);

        let mut uninit_buf = alloc_uninit_bytes(m);

        let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
        crate::multiversion::encode::auto(src, dst, base32.kind, base32.padding);

        let len = uninit_buf.len();
        let ptr = Box::into_raw(uninit_buf).cast::<u8>();
        Box::from_raw(core::str::from_utf8_unchecked_mut(slice_mut(ptr, len)))
    }
}

fn encode_append_vec(base32: &Base32, src: &[u8], buf: &mut Vec<u8>) {
    if src.is_empty() {
        return;
    }

    let m = encoded_length_unchecked(src.len(), base32.padding);
    assert!(m <= usize::MAX / 2);

    buf.reserve_exact(m);
    let prev_len = buf.len();

    unsafe {
        let dst = buf.as_mut_ptr().add(prev_len);
        crate::multiversion::encode::auto(src, dst, base32.kind, base32.padding);

        buf.set_len(prev_len + m);
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Decode for Box<[u8]> {
    #[inline]
    fn from_base32_decode(base32: &Base32, data: &[u8]) -> Result<Self, Error> {
        decode_to_boxed_bytes(base32, data)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Decode for Vec<u8> {
    #[inline]
    fn from_base32_decode(base32: &Base32, data: &[u8]) -> Result<Self, Error> {
        let ans = <Box<[u8]> as FromBase32Decode>::from_base32_decode(base32, data)?;
        Ok(Vec::from(ans))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Encode for Box<[u8]> {
    #[inline]
    fn from_base32_encode(base32: &Base32, data: &[u8]) -> Self {
        let ans = encode_to_boxed_str(base32, data);
        ans.into_boxed_bytes()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Encode for Box<str> {
    #[inline]
    fn from_base32_encode(base32: &Base32, data: &[u8]) -> Self {
        encode_to_boxed_str(base32, data)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Encode for Vec<u8> {
    #[inline]
    fn from_base32_encode(base32: &Base32, data: &[u8]) -> Self {
        let ans = encode_to_boxed_str(base32, data);
        Vec::from(ans.into_boxed_bytes())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl FromBase32Encode for String {
    #[inline]
    fn from_base32_encode(base32: &Base32, data: &[u8]) -> Self {
        let ans = encode_to_boxed_str(base32, data);
        String::from(ans)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl AppendBase32Encode for Vec<u8> {
    #[inline]
    fn append_base32_encode(base32: &Base32, src: &[u8], dst: &mut Self) {
        encode_append_vec(base32, src, dst);
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl AppendBase32Encode for String {
    #[inline]
    fn append_base32_encode(base32: &Base32, src: &[u8], dst: &mut Self) {
        unsafe { encode_append_vec(base32, src, dst.as_mut_vec()) };
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl AppendBase32Decode for Vec<u8> {
    #[inline]
    fn append_base32_decode(base32: &Base32, src: &[u8], dst: &mut Self) -> Result<(), Error> {
        decode_append_vec(base32, src, dst)
    }
}
