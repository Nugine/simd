use crate::Error;

use vsimd::base64::{STANDARD_ALSW_LUT, URL_SAFE_ALSW_LUT};
use vsimd::base64::{STANDARD_CHARSET, URL_SAFE_CHARSET};
use vsimd::base64::{STANDARD_ENCODING_SHIFT, URL_SAFE_ENCODING_SHIFT};

use vsimd::tools::{read, slice_mut, write};
use vsimd::{item_group, SIMD256};

use outref::OutRef;

#[cfg(feature = "alloc")]
item_group!(
    use alloc::boxed::Box;
    use vsimd::tools::{alloc_uninit_bytes, assume_init};
);

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Standard,
    UrlSafe,
}

#[inline(always)]
const fn encoded_length_unchecked(len: usize, padding: bool) -> usize {
    let extra = len % 3;
    if extra == 0 {
        len / 3 * 4
    } else if padding {
        len / 3 * 4 + 4
    } else {
        len / 3 * 4 + extra + 1
    }
}

#[inline(always)]
unsafe fn encode_bits24(src: *const u8, dst: *mut u8, charset: *const u8) {
    let x = u32::from_be_bytes([0, read(src, 0), read(src, 1), read(src, 2)]);
    for i in 0..4 {
        let y = read(charset, ((x >> (18 - i * 6)) & 0x3f) as usize);
        write(dst, i, y);
    }
}

#[inline(always)]
unsafe fn encode_bits48(src: *const u8, dst: *mut u8, charset: *const u8) {
    let x = u64::from_be_bytes(src.cast::<[u8; 8]>().read());
    for i in 0..8 {
        let y = read(charset, ((x >> (58 - i * 6)) & 0x3f) as usize);
        write(dst, i, y)
    }
}

#[inline(always)]
unsafe fn encode_extra(extra: usize, src: *const u8, dst: *mut u8, charset: *const u8, padding: bool) {
    match extra {
        0 => {}
        1 => {
            let x = read(src, 0);
            let y1 = read(charset, (x >> 2) as usize);
            let y2 = read(charset, ((x << 6) >> 2) as usize);
            write(dst, 0, y1);
            write(dst, 1, y2);
            if padding {
                write(dst, 2, b'=');
                write(dst, 3, b'=');
            }
        }
        2 => {
            let x1 = read(src, 0);
            let x2 = read(src, 1);
            let y1 = read(charset, (x1 >> 2) as usize);
            let y2 = read(charset, (((x1 << 6) >> 2) | (x2 >> 4)) as usize);
            let y3 = read(charset, ((x2 << 4) >> 2) as usize);
            write(dst, 0, y1);
            write(dst, 1, y2);
            write(dst, 2, y3);
            if padding {
                write(dst, 3, b'=');
            }
        }
        _ => core::hint::unreachable_unchecked(),
    }
}

pub unsafe fn encode_fallback(src: &[u8], mut dst: *mut u8, kind: Kind, padding: bool) {
    let charset = match kind {
        Kind::Standard => STANDARD_CHARSET.as_ptr(),
        Kind::UrlSafe => URL_SAFE_CHARSET.as_ptr(),
    };

    let (mut src, mut len) = (src.as_ptr(), src.len());

    while len >= (6 + 2) {
        encode_bits48(src, dst, charset);
        src = src.add(6);
        dst = dst.add(8);
        len -= 6;
    }

    while len >= 3 {
        encode_bits24(src, dst, charset);
        src = src.add(3);
        dst = dst.add(4);
        len -= 3;
    }

    encode_extra(len, src, dst, charset, padding)
}

pub unsafe fn encode_simd<S: SIMD256>(s: S, src: &[u8], mut dst: *mut u8, kind: Kind, padding: bool) {
    let (charset, shift_lut) = match kind {
        Kind::Standard => (STANDARD_CHARSET.as_ptr(), STANDARD_ENCODING_SHIFT),
        Kind::UrlSafe => (URL_SAFE_CHARSET.as_ptr(), URL_SAFE_ENCODING_SHIFT),
    };

    let (mut src, mut len) = (src.as_ptr(), src.len());

    if len >= (6 + 24 + 4) {
        for _ in 0..2 {
            encode_bits24(src, dst, charset);
            src = src.add(3);
            dst = dst.add(4);
            len -= 3;
        }

        while len >= (24 + 4) {
            let x = s.v256_load_unaligned(src.sub(4));
            let y = vsimd::base64::encode_bytes24(s, x, shift_lut);
            s.v256_store_unaligned(dst, y);
            src = src.add(24);
            dst = dst.add(32);
            len -= 24;
        }
    }

    let src = core::slice::from_raw_parts(src, len);
    encode_fallback(src, dst, kind, padding)
}

const fn decode_table(charset: &'static [u8; 64]) -> [u8; 256] {
    let mut table = [0xff; 256];
    let mut i = 0;
    while i < charset.len() {
        table[charset[i] as usize] = i as u8;
        i += 1;
    }
    table
}

const STANDARD_DECODE_TABLE: &[u8; 256] = &decode_table(STANDARD_CHARSET);
const URL_SAFE_DECODE_TABLE: &[u8; 256] = &decode_table(URL_SAFE_CHARSET);

#[inline(always)]
fn decoded_length(src: &[u8], padding: bool) -> Result<(usize, usize), Error> {
    if src.is_empty() {
        return Ok((0, 0));
    }

    let n = unsafe {
        let len = src.len();
        if padding {
            ensure!(len % 4 == 0);
            let last1 = *src.get_unchecked(len - 1);
            let last2 = *src.get_unchecked(len - 2);
            let count = (last1 == b'=') as usize + (last2 == b'=') as usize;
            len - count
        } else {
            len
        }
    };

    let m = match n % 4 {
        0 => n / 4 * 3,
        1 => return Err(Error::new()),
        2 => n / 4 * 3 + 1,
        3 => n / 4 * 3 + 2,
        _ => unsafe { core::hint::unreachable_unchecked() },
    };

    Ok((n, m))
}

#[inline(always)]
unsafe fn decode_ascii8(src: *const u8, dst: *mut u8, table: *const u8) -> Result<(), Error> {
    let mut x = u64::from_le_bytes(src.cast::<[u8; 8]>().read());
    let mut y: u64 = 0;
    let mut flag = 0;
    for i in 0..8 {
        let bits = read(table, (x & 0xff) as usize);
        flag |= bits;
        x >>= 8;
        y |= (bits as u64) << (58 - i * 6);
    }
    ensure!(flag != 0xff);
    dst.cast::<u64>().write_unaligned(y.to_be());
    Ok(())
}

#[inline(always)]
unsafe fn decode_ascii4(src: *const u8, dst: *mut u8, table: *const u8) -> Result<(), Error> {
    let mut x = u32::from_le_bytes(src.cast::<[u8; 4]>().read());
    let mut y: u32 = 0;
    let mut flag = 0;
    for i in 0..4 {
        let bits = read(table, (x & 0xff) as usize);
        flag |= bits;
        x >>= 8;
        y |= (bits as u32) << (18 - i * 6);
    }
    ensure!(flag != 0xff);
    let y = y.to_be_bytes();
    write(dst, 0, y[1]);
    write(dst, 1, y[2]);
    write(dst, 2, y[3]);
    Ok(())
}

#[inline(always)]
unsafe fn decode_extra(extra: usize, src: *const u8, dst: *mut u8, table: *const u8) -> Result<(), Error> {
    match extra {
        0 => {}
        1 => core::hint::unreachable_unchecked(),
        2 => {
            let [x1, x2] = src.cast::<[u8; 2]>().read();
            let y1 = read(table, x1 as usize);
            let y2 = read(table, x2 as usize);
            ensure!(y2 & 0x0f == 0 && (y1 | y2) != 0xff);
            write(dst, 0, (y1 << 2) | (y2 >> 4));
        }
        3 => {
            let [x1, x2, x3] = src.cast::<[u8; 3]>().read();
            let y1 = read(table, x1 as usize);
            let y2 = read(table, x2 as usize);
            let y3 = read(table, x3 as usize);
            ensure!(y3 & 0x03 == 0 && (y1 | y2 | y3) != 0xff);
            write(dst, 0, (y1 << 2) | (y2 >> 4));
            write(dst, 1, (y2 << 4) | (y3 >> 2));
        }
        _ => core::hint::unreachable_unchecked(),
    }
    Ok(())
}

pub unsafe fn decode_fallback(mut src: *const u8, mut dst: *mut u8, mut n: usize, kind: Kind) -> Result<(), Error> {
    let table = match kind {
        Kind::Standard => STANDARD_DECODE_TABLE.as_ptr(),
        Kind::UrlSafe => URL_SAFE_DECODE_TABLE.as_ptr(),
    };

    // n*3/4 >= 6+2
    while n >= 11 {
        decode_ascii8(src, dst, table)?;
        src = src.add(8);
        dst = dst.add(6);
        n -= 8;
    }

    while n >= 4 {
        decode_ascii4(src, dst, table)?;
        src = src.add(4);
        dst = dst.add(3);
        n -= 4;
    }

    decode_extra(n, src, dst, table)
}

pub unsafe fn decode_simd<S: SIMD256>(
    s: S,
    mut src: *const u8,
    mut dst: *mut u8,
    mut n: usize,
    kind: Kind,
) -> Result<(), Error> {
    let lut = match kind {
        Kind::Standard => STANDARD_ALSW_LUT,
        Kind::UrlSafe => URL_SAFE_ALSW_LUT,
    };

    // n*3/4 >= 24+4
    while n >= 38 {
        let x = s.v256_load_unaligned(src);
        let y = vsimd::base64::decode_ascii32(s, x, lut).map_err(|()| Error::new())?;

        let (y1, y2) = y.to_v128x2();
        s.v128_store_unaligned(dst, y1);
        s.v128_store_unaligned(dst.add(12), y2);

        src = src.add(32);
        dst = dst.add(24);
        n -= 32;
    }

    decode_fallback(src, dst, n, kind)
}

mod multiversion {
    use super::*;

    use vsimd::simd_dispatch;

    simd_dispatch!(
        name        = encode,
        signature   = fn(src: &[u8], dst: *mut u8, kind: Kind, padding: bool) -> (),
        fallback    = {encode_fallback},
        simd        = {encode_simd},
        safety      = {unsafe},
    );

    simd_dispatch!(
        name        = decode,
        signature   = fn(src: *const u8, dst: *mut u8, n: usize, kind: Kind) -> Result<(), Error>,
        fallback    = {decode_fallback},
        simd        = {decode_simd},
        safety      = {unsafe},
    );
}

/// RFC4648 Base64 variants
///
/// + [`STANDARD`](crate::STANDARD)
/// + [`URL_SAFE`](crate::URL_SAFE)
///
#[derive(Debug)]
pub struct Rfc4648Base64 {
    kind: Kind,
    padding: bool,
}

/// Standard charset with padding.
pub const STANDARD: Rfc4648Base64 = Rfc4648Base64 {
    kind: Kind::Standard,
    padding: true,
};

/// URL-safe charset with padding.
pub const URL_SAFE: Rfc4648Base64 = Rfc4648Base64 {
    kind: Kind::UrlSafe,
    padding: true,
};

impl Rfc4648Base64 {
    /// Sets whether to handle padding.
    #[inline(always)]
    #[must_use]
    pub const fn padding(mut self, padding: bool) -> Self {
        self.padding = padding;
        self
    }

    /// Returns the character set.
    #[inline]
    #[must_use]
    pub const fn charset(&self) -> &[u8; 64] {
        match self.kind {
            Kind::Standard => STANDARD_CHARSET,
            Kind::UrlSafe => URL_SAFE_CHARSET,
        }
    }

    /// Calculates the encoded length.
    ///
    /// # Panics
    /// This function will panics if `n > isize::MAX`.
    #[inline]
    #[must_use]
    pub const fn encoded_length(&self, n: usize) -> usize {
        assert!(n <= usize::MAX / 2);
        encoded_length_unchecked(n, self.padding)
    }

    /// Estimates the decoded length.
    ///
    /// The result is an upper bound which can be used for allocation.
    #[inline]
    #[must_use]
    pub const fn estimated_decoded_length(&self, n: usize) -> usize {
        if n % 4 == 0 {
            n / 4 * 3
        } else {
            (n / 4 + 1) * 3
        }
    }

    /// Calculates the decoded length.
    ///
    /// The result is a precise value which can be used for allocation.
    #[inline]
    pub fn decoded_length(&self, data: &[u8]) -> Result<usize, Error> {
        let (_, m) = decoded_length(data, self.padding)?;
        Ok(m)
    }

    /// Encodes `src` and writes to `dst`.
    ///
    /// # Panics
    /// This function will panic if the length of `dst` is not enough.
    #[inline]
    #[must_use]
    pub fn encode<'s, 'd>(&'_ self, src: &'s [u8], mut dst: OutRef<'d, [u8]>) -> &'d mut [u8] {
        unsafe {
            let m = encoded_length_unchecked(src.len(), self.padding);
            assert!(dst.len() >= m);

            let dst = dst.as_mut_ptr();
            self::multiversion::encode::auto_indirect(src, dst, self.kind, self.padding);

            slice_mut(dst, m)
        }
    }
    /// Encodes `src` to `dst` and returns [`&mut str`](str).
    ///
    /// # Panics
    /// This function will panic if the length of `dst` is not enough.
    #[inline]
    #[must_use]
    pub fn encode_as_str<'s, 'd>(&'_ self, src: &'s [u8], dst: OutRef<'d, [u8]>) -> &'d mut str {
        let ans = self.encode(src, dst);
        unsafe { core::str::from_utf8_unchecked_mut(ans) }
    }

    /// Decodes `src` and writes to `dst`.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `src` is invalid.
    ///
    /// # Panics
    /// This function will panic if the length of `dst` is not enough.
    #[inline]
    pub fn decode<'s, 'd>(&'_ self, src: &'s [u8], mut dst: OutRef<'d, [u8]>) -> Result<&'d mut [u8], Error> {
        unsafe {
            let (n, m) = decoded_length(src, self.padding)?;
            assert!(dst.len() >= m);

            let src = src.as_ptr();
            let dst = dst.as_mut_ptr();
            self::multiversion::decode::auto_indirect(src, dst, n, self.kind)?;

            Ok(slice_mut(dst, m))
        }
    }

    /// Decodes `data` and writes inplace.
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    #[inline]
    pub fn decode_inplace<'d>(&'_ self, data: &'d mut [u8]) -> Result<&'d mut [u8], Error> {
        unsafe {
            let (n, m) = decoded_length(data, self.padding)?;

            let dst: *mut u8 = data.as_mut_ptr();
            let src: *const u8 = dst;
            self::multiversion::decode::auto_indirect(src, dst, n, self.kind)?;

            Ok(slice_mut(dst, m))
        }
    }

    /// Encodes `data` and returns [`Box<str>`]
    ///
    /// # Panics
    /// This function will panics if the encoded length of `data` is greater than `isize::MAX`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    #[inline]
    #[must_use]
    pub fn encode_to_boxed_str(&self, data: &[u8]) -> Box<str> {
        if data.is_empty() {
            return Box::from("");
        }

        unsafe {
            let m = encoded_length_unchecked(data.len(), self.padding);
            assert!(m <= usize::MAX / 2);

            let mut uninit_buf = alloc_uninit_bytes(m);

            let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
            self::multiversion::encode::auto_indirect(data, dst, self.kind, self.padding);

            let len = uninit_buf.len();
            let ptr = Box::into_raw(uninit_buf).cast::<u8>();
            Box::from_raw(core::str::from_utf8_unchecked_mut(slice_mut(ptr, len)))
        }
    }

    /// Decodes `data` and returns [`Box<[u8]>`](Box)
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn decode_to_boxed_bytes(&self, data: &[u8]) -> Result<Box<[u8]>, Error> {
        if data.is_empty() {
            return Ok(Box::from([]));
        }

        unsafe {
            let (n, m) = decoded_length(data, self.padding)?;

            // safety: 0 < m < isize::MAX
            let mut uninit_buf = alloc_uninit_bytes(m);

            let dst: *mut u8 = uninit_buf.as_mut_ptr().cast();
            let src: *const u8 = data.as_ptr();
            self::multiversion::decode::auto_indirect(src, dst, n, self.kind)?;

            Ok(assume_init(uninit_buf))
        }
    }

    /// Forgiving decodes `data` and writes inplace.
    ///
    /// See <https://infra.spec.whatwg.org/#forgiving-base64>
    ///
    /// # Errors
    /// This function returns `Err` if the content of `data` is invalid.
    #[inline]
    pub fn forgiving_decode_inplace(data: &mut [u8]) -> Result<&mut [u8], Error> {
        let data = crate::forgiving::normalize(data);
        STANDARD.padding(false).decode_inplace(data)
    }
}
