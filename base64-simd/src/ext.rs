use crate::{Base64, Error, OutBuf};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
use simd_abstraction::tools::alloc_uninit_bytes;

impl Base64 {
    /// Encodes `src` and returns [`Box<str>`]
    ///
    /// # Panics
    /// This function panics if:
    ///
    /// + The encoded length of `src` is greater than `isize::MAX`
    ///
    #[cfg(feature = "alloc")]
    pub fn encode_to_boxed_str(&self, src: &[u8]) -> Box<str> {
        use core::{slice, str};

        if src.is_empty() {
            return Box::from("");
        }

        unsafe {
            let m = Self::encoded_length_unchecked(src.len(), self.padding);
            assert!(m <= (isize::MAX as usize));
            let mut uninit_buf = alloc_uninit_bytes(m);
            Self::encode(self, src, OutBuf::from_uninit_mut(&mut *uninit_buf)).unwrap();

            let ptr = Box::into_raw(uninit_buf).cast::<u8>();
            let buf = slice::from_raw_parts_mut(ptr, m);
            Box::from_raw(str::from_utf8_unchecked_mut(buf))
        }
    }

    /// Decodes `src` and returns [`Box<[u8]>`](Box)
    ///
    /// # Errors
    /// This function returns `Err` if:
    ///
    /// + The content of `src` is invalid.
    ///
    #[cfg(feature = "alloc")]
    pub fn decode_to_boxed_bytes(&self, src: &[u8]) -> Result<Box<[u8]>, Error> {
        use core::slice;

        if src.is_empty() {
            return Ok(Box::from([]));
        }
        unsafe {
            let (_, m) = Self::decoded_length_unchecked(src, self.padding)?;

            // safety: 0 < m < isize::MAX
            let mut uninit_buf = alloc_uninit_bytes(m);
            Self::decode(self, src, OutBuf::from_uninit_mut(&mut *uninit_buf))?;

            let ptr = Box::into_raw(uninit_buf).cast::<u8>();
            let buf = slice::from_raw_parts_mut(ptr, m);
            Ok(Box::from_raw(buf))
        }
    }
}
