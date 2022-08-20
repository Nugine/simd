use crate::common::{decode_bits, encode_bits, read_be_bytes, write_be_bytes};
use crate::error::Error;

use simd_abstraction::tools::{slice_mut, write};
use simd_abstraction::OutBuf;

const BASE32_CHARSET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

const BASE32HEX_CHARSET: &[u8; 32] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

const fn decoding_table(charset: &[u8; 32]) -> [u8; 256] {
    let mut table = [0xff; 256];
    let mut i = 0;
    while i < 32 {
        let x = charset[i];
        table[x as usize] = x;
        i += 1;
    }
    table
}

const BASE32_TABLE: &[u8; 256] = &decoding_table(BASE32_CHARSET);

const BASE32HEX_TABLE: &[u8; 256] = &decoding_table(BASE32HEX_CHARSET);

unsafe fn encode_extra(src: *const u8, extra: usize, dst: *mut u8, charset: *const u8, padding: bool) {
    match extra {
        0 => {}
        1 => {
            let u10 = read_be_bytes::<1>(src) << 2;
            encode_bits::<2>(dst, charset, u10);
            if padding {
                (2..8).for_each(|i| write(dst, i, b'='));
            }
        }
        2 => {
            let u20 = read_be_bytes::<2>(src) << 4;
            encode_bits::<4>(dst, charset, u20);
            if padding {
                (4..8).for_each(|i| write(dst, i, b'='));
            }
        }
        3 => {
            let u25 = read_be_bytes::<3>(src) << 1;
            encode_bits::<5>(dst, charset, u25);
            if padding {
                (5..8).for_each(|i| write(dst, i, b'='));
            }
        }
        4 => {
            let u35 = read_be_bytes::<4>(src) << 3;
            encode_bits::<7>(dst, charset, u35);
            if padding {
                write(dst, 7, b'=');
            }
        }
        _ => core::hint::unreachable_unchecked(),
    }
}

unsafe fn encode_fallback(src: &[u8], mut dst: *mut u8, charset: &[u8; 32], padding: bool) {
    let charset: *const u8 = charset.as_ptr();
    let len = src.len();
    let mut src = src.as_ptr();
    let end = src.add(len / 5 * 5);

    while src < end {
        let u40 = read_be_bytes::<5>(src);
        encode_bits::<8>(dst, charset, u40);
        src = src.add(5);
        dst = dst.add(8);
    }

    encode_extra(src, len % 5, dst, charset, padding)
}

unsafe fn decode_extra(src: *const u8, extra: usize, dst: *mut u8, table: *const u8) -> Result<(), Error> {
    match extra {
        0 => {}
        2 => {
            let u10 = decode_bits::<2>(src, table)?;
            ensure!(u10 & 0b11 == 0);
            write_be_bytes::<1>(dst, u10 >> 2);
        }
        4 => {
            let u20 = decode_bits::<4>(src, table)?;
            ensure!(u20 & 0b1111 == 0);
            write_be_bytes::<2>(dst, u20 >> 4);
        }
        5 => {
            let u25 = decode_bits::<5>(src, table)?;
            ensure!(u25 & 0b1 == 0);
            write_be_bytes::<3>(dst, u25 >> 1);
        }
        7 => {
            let u35 = decode_bits::<7>(src, table)?;
            ensure!(u35 & 0b111 == 0);
            write_be_bytes::<4>(dst, u35 >> 3);
        }
        _ => core::hint::unreachable_unchecked(),
    }
    Ok(())
}

unsafe fn decode_fallback(mut src: *const u8, len: usize, mut dst: *mut u8, table: &[u8; 256]) -> Result<(), Error> {
    let table = table.as_ptr();
    let end = src.add(len / 8 * 8);

    while src < end {
        let u40 = decode_bits::<8>(src, table)?;
        write_be_bytes::<5>(dst, u40);
        src = src.add(8);
        dst = dst.add(5);
    }

    decode_extra(src, len % 8, dst, table)
}

fn encoded_length_unchecked(len: usize, padding: bool) -> usize {
    let l = len / 5 * 8;
    if len % 5 == 0 {
        return l;
    }
    if padding {
        return l + 8;
    }
    const EXTRA: [u8; 5] = [0, 2, 4, 5, 7];
    l + EXTRA[len % 5] as usize
}

fn decoded_length(data: &[u8], padding: bool) -> Result<(usize, usize), Error> {
    if data.is_empty() {
        return Ok((0, 0));
    }

    let len = data.len();
    let n = if padding {
        ensure!(len % 8 == 0);
        let last = unsafe { data.get_unchecked(len - 6..) };
        let count = last.iter().copied().filter(|&x| x == b'=').count();
        len - count
    } else {
        data.len()
    };

    const EXTRA: [u8; 8] = [0, 0xff, 1, 0xff, 2, 3, 0xff, 4];
    let extra = EXTRA[n % 8];
    ensure!(extra != 0xff);
    let m = n / 8 * 5 + extra as usize;
    Ok((n, m))
}

/// TODO
#[derive(Debug)]
pub struct Rfc4648Base32 {
    charset: &'static [u8; 32],
    table: &'static [u8; 256],
    padding: bool,
}

/// TODO
pub const BASE32: Rfc4648Base32 = Rfc4648Base32 {
    charset: BASE32_CHARSET,
    table: BASE32_TABLE,
    padding: true,
};

/// TODO
pub const BASE32HEX: Rfc4648Base32 = Rfc4648Base32 {
    charset: BASE32HEX_CHARSET,
    table: BASE32HEX_TABLE,
    padding: true,
};

impl Rfc4648Base32 {
    /// TODO
    #[inline]
    pub const fn padding(mut self, padding: bool) -> Self {
        self.padding = padding;
        self
    }

    /// TODO
    #[inline]
    pub fn encoded_length(&self, n: usize) -> usize {
        assert!(n < usize::MAX / 2);
        encoded_length_unchecked(n, self.padding)
    }

    /// TODO
    #[inline]
    pub fn decoded_length(&self, data: &[u8]) -> Result<usize, Error> {
        let (_, m) = decoded_length(data, self.padding)?;
        Ok(m)
    }

    /// TODO
    #[inline]
    pub const fn estimated_decoded_length(&self, n: usize) -> usize {
        if n % 8 == 0 {
            n / 8 * 5
        } else {
            (n / 8 + 1) * 5
        }
    }

    /// TODO
    #[inline]
    pub fn encode<'s, 'd>(&'_ self, src: &'s [u8], mut dst: OutBuf<'d, u8>) -> &'d mut [u8] {
        let encoded_len = encoded_length_unchecked(src.len(), self.padding);
        assert!(dst.len() >= encoded_len);

        unsafe {
            let dst = dst.as_mut_ptr();
            encode_fallback(src, dst, self.charset, self.padding);

            slice_mut(dst, encoded_len)
        }
    }

    /// TODO
    #[inline]
    pub fn decode<'s, 'd>(&'_ self, src: &'s [u8], mut dst: OutBuf<'d, u8>) -> Result<&'d mut [u8], Error> {
        let (data_len, decoded_len) = decoded_length(src, self.padding)?;
        assert!(dst.len() >= decoded_len);

        unsafe {
            let src = src.as_ptr();
            let dst = dst.as_mut_ptr();
            decode_fallback(src, data_len, dst, self.table)?;

            Ok(slice_mut(dst, decoded_len))
        }
    }
}
