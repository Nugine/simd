use crate::common::{decode_bits, encode_bits, read_be_bytes, write_be_bytes};
use crate::error::Error;

use simd_abstraction::tools::slice_mut;
use simd_abstraction::OutBuf;

const CHARSET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

const fn decoding_table(case_insensitive: bool) -> [u8; 256] {
    let mut table = [0xff; 256];
    let mut i = 0;
    while i < 32 {
        let x = CHARSET[i];
        table[x as usize] = x;
        if case_insensitive {
            table[x.to_ascii_lowercase() as usize] = x;
        }
        i += 1;
    }
    table
}

const INSENSITIVE_TABLE: &[u8; 256] = &decoding_table(true);

const SENSITIVE_TABLE: &[u8; 256] = &decoding_table(false);

unsafe fn decode_extra(src: *const u8, extra: usize, dst: *mut u8, table: *const u8) -> Result<u8, Error> {
    match extra {
        0 => Ok(0),
        2 => {
            let u10 = decode_bits::<2>(src, table)?;
            ensure!(u10 >> 8 == 0);
            write_be_bytes::<1>(dst, u10);
            Ok(1)
        }
        4 => {
            let u20 = decode_bits::<4>(src, table)?;
            ensure!(u20 >> 16 == 0);
            write_be_bytes::<2>(dst, u20);
            Ok(2)
        }
        5 => {
            let u25 = decode_bits::<5>(src, table)?;
            ensure!(u25 >> 24 == 0);
            write_be_bytes::<3>(dst, u25);
            Ok(3)
        }
        7 => {
            let u35 = decode_bits::<7>(src, table)?;
            ensure!(u35 >> 32 == 0);
            write_be_bytes::<4>(dst, u35);
            Ok(4)
        }
        _ => Err(Error::new()),
    }
}

unsafe fn decode_fallback(mut src: *const u8, len: usize, mut dst: *mut u8, table: &[u8; 256]) -> Result<(), Error> {
    let table = table.as_ptr();
    let end = src.add(len);

    let extra = len % 8;
    let written = decode_extra(src, extra, dst, table)?;
    src = src.add(extra);
    dst = dst.add(written as usize);

    while src < end {
        let u40 = decode_bits::<8>(src, table)?;
        write_be_bytes::<5>(dst, u40);
        src = src.add(8);
        dst = dst.add(5);
    }

    Ok(())
}

unsafe fn encode_extra(src: *const u8, extra: usize, dst: *mut u8) -> u8 {
    let charset: *const u8 = CHARSET.as_ptr();
    match extra {
        0 => 0,
        1 => {
            let u10 = read_be_bytes::<1>(src);
            encode_bits::<2>(dst, charset, u10);
            2
        }
        2 => {
            let u20 = read_be_bytes::<2>(src);
            encode_bits::<4>(dst, charset, u20);
            4
        }
        3 => {
            let u25 = read_be_bytes::<3>(src);
            encode_bits::<5>(dst, charset, u25);
            5
        }
        4 => {
            let u35 = read_be_bytes::<4>(src);
            encode_bits::<7>(dst, charset, u35);
            7
        }
        _ => core::hint::unreachable_unchecked(),
    }
}

unsafe fn encode_fallback(src: &[u8], mut dst: *mut u8) {
    let charset: *const u8 = CHARSET.as_ptr();
    let len = src.len();
    let mut src = src.as_ptr();
    let end = src.add(len);

    let extra = len % 5;
    let written = encode_extra(src, extra, dst);
    src = src.add(extra);
    dst = dst.add(written as usize);

    while src < end {
        let u40 = read_be_bytes::<5>(src);
        encode_bits::<8>(dst, charset, u40);
        src = src.add(5);
        dst = dst.add(8);
    }
}

fn encoded_length_unchecked(len: usize) -> usize {
    const EXTRA: [u8; 5] = [0, 2, 4, 5, 7];
    len / 5 * 8 + EXTRA[len % 5] as usize
}

fn decoded_length(len: usize) -> Result<usize, Error> {
    const EXTRA: [u8; 8] = [0, 0xff, 1, 0xff, 2, 3, 0xff, 4];
    let extra = EXTRA[len % 8];
    ensure!(extra != 0xff);
    Ok(len / 8 * 5 + extra as usize)
}

/// TODO
#[derive(Debug)]
pub struct CrockfordBase32 {
    case_insensitive: bool,
}

/// TODO
pub const CROCKFORD_BASE32: CrockfordBase32 = CrockfordBase32 {
    case_insensitive: false,
};

impl CrockfordBase32 {
    /// TODO
    #[inline]
    pub const fn case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }

    /// TODO
    #[inline]
    pub fn encoded_length(&self, n: usize) -> usize {
        assert!(n < usize::MAX / 2);
        encoded_length_unchecked(n)
    }

    /// TODO
    #[inline]
    pub fn decoded_length(&self, n: usize) -> Result<usize, Error> {
        decoded_length(n)
    }

    /// TODO
    #[inline]
    pub fn encode<'s, 'd>(&'_ self, src: &'s [u8], mut dst: OutBuf<'d, u8>) -> &'d mut [u8] {
        let encoded_len = encoded_length_unchecked(src.len());
        assert!(dst.len() >= encoded_len);

        unsafe {
            let dst = dst.as_mut_ptr();
            encode_fallback(src, dst);

            slice_mut(dst, encoded_len)
        }
    }

    /// TODO
    #[inline]
    pub fn decode<'s, 'd>(&'_ self, src: &'s [u8], mut dst: OutBuf<'d, u8>) -> Result<&'d mut [u8], Error> {
        let data_len = src.len();
        let decoded_len = decoded_length(data_len)?;
        assert!(dst.len() >= decoded_len);

        let table = if self.case_insensitive {
            INSENSITIVE_TABLE
        } else {
            SENSITIVE_TABLE
        };

        unsafe {
            let src = src.as_ptr();
            let dst = dst.as_mut_ptr();
            decode_fallback(src, data_len, dst, table)?;

            Ok(slice_mut(dst, decoded_len))
        }
    }
}
