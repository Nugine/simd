use crate::{Base64, Error, OutBuf};

use core::ops::Not;

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
    #[inline]
    pub fn encode_to_boxed_str(&self, src: &[u8]) -> Box<str> {
        use core::{slice, str};

        if src.is_empty() {
            return Box::from("");
        }

        unsafe {
            let m = Self::encoded_length_unchecked(src.len(), self.padding);
            assert!(m <= (isize::MAX as usize));
            let mut uninit_buf = alloc_uninit_bytes(m);
            Self::encode(self, src, OutBuf::uninit(&mut *uninit_buf)).unwrap();

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
    #[inline]
    pub fn decode_to_boxed_bytes(&self, src: &[u8]) -> Result<Box<[u8]>, Error> {
        use core::slice;

        if src.is_empty() {
            return Ok(Box::from([]));
        }
        unsafe {
            let (_, m) = Self::decoded_length_unchecked(src, self.padding)?;

            // safety: 0 < m < isize::MAX
            let mut uninit_buf = alloc_uninit_bytes(m);
            Self::decode(self, src, OutBuf::uninit(&mut *uninit_buf))?;

            let ptr = Box::into_raw(uninit_buf).cast::<u8>();
            let buf = slice::from_raw_parts_mut(ptr, m);
            Ok(Box::from_raw(buf))
        }
    }

    /// Forgiving decodes `buf` and writes inplace.
    ///
    /// See <https://infra.spec.whatwg.org/#forgiving-base64>
    #[inline]
    pub fn forgiving_decode_inplace(buf: &mut [u8]) -> Result<&mut [u8], Error> {
        let buf = forgiving_fix_data(buf);
        Self::STANDARD_NO_PAD.decode_inplace(buf)
    }
}

fn remove_ascii_whitespace(buf: &mut [u8]) -> &mut [u8] {
    unsafe {
        let mut src = buf.as_ptr();
        let end = src.add(buf.len());

        while src < end {
            if src.read().is_ascii_whitespace() {
                break;
            }
            src = src.add(1);
        }

        let mut dst = src as *mut u8;
        while src < end {
            let byte = src.read();
            if byte.is_ascii_whitespace().not() {
                dst.write(byte);
                dst = dst.add(1);
            }
            src = src.add(1);
        }

        let len = dst.offset_from(buf.as_ptr()) as usize;
        buf.get_unchecked_mut(..len)
    }
}

const fn forgiving_discard_table(mask: u8) -> [u8; 256] {
    let charset = crate::fallback::STANDARD_CHARSET;
    let mut table = [0; 256];

    let mut i = 0;
    loop {
        table[i as usize] = i;
        if i == 255 {
            break;
        }
        i += 1;
    }

    let mut i = 0;
    while i < 64 {
        table[charset[i] as usize] = charset[i & mask as usize];
        i += 1;
    }
    table
}

#[inline(always)]
fn forgiving_discard4(ch: &mut u8) {
    const TABLE: &[u8; 256] = &forgiving_discard_table(0xf0);
    unsafe { *ch = *TABLE.get_unchecked(*ch as usize) }
}

#[inline(always)]
fn forgiving_discard2(ch: &mut u8) {
    const TABLE: &[u8; 256] = &forgiving_discard_table(0xfc);
    unsafe { *ch = *TABLE.get_unchecked(*ch as usize) }
}

fn forgiving_fix_data(buf: &mut [u8]) -> &mut [u8] {
    let buf = remove_ascii_whitespace(buf);

    if buf.is_empty() {
        return buf;
    }

    unsafe {
        let len = buf.len();
        match len % 4 {
            0 => {
                let x1 = *buf.get_unchecked(len - 1);
                let x2 = *buf.get_unchecked(len - 2);
                if x1 == b'=' {
                    if x2 == b'=' {
                        let last3 = buf.get_unchecked_mut(len - 3);
                        forgiving_discard4(last3);
                        buf.get_unchecked_mut(..len - 2)
                    } else {
                        let last2 = buf.get_unchecked_mut(len - 2);
                        forgiving_discard2(last2);
                        buf.get_unchecked_mut(..len - 1)
                    }
                } else {
                    buf
                }
            }
            1 => buf,
            2 => {
                let last1 = buf.get_unchecked_mut(len - 1);
                forgiving_discard4(last1);
                buf
            }
            3 => {
                let last1 = buf.get_unchecked_mut(len - 1);
                forgiving_discard2(last1);
                buf
            }
            _ => core::hint::unreachable_unchecked(),
        }
    }
}

#[test]
fn test_forgiving() {
    let inputs = ["ab", "abc", "abcd"];
    let outputs: &[&[u8]] = &[&[105], &[105, 183], &[105, 183, 29]];

    for i in 0..inputs.len() {
        let (src, expected) = (inputs[i], outputs[i]);
        let mut buf = src.to_owned().into_bytes();
        let ans = Base64::forgiving_decode_inplace(&mut buf).unwrap();
        assert_eq!(ans, expected, "src = {:?}, expected = {:?}", src, expected);
    }
}
