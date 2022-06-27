#![allow(missing_docs)]

use crate::utils::{empty_slice_mut, read, write};
use crate::{Base64, Base64Kind, Error, OutBuf, ERROR};

use core::slice;

pub(crate) const STANDARD_CHARSET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(crate) const URL_SAFE_CHARSET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

const fn decode_table(charset: &'static [u8; 64]) -> [u8; 256] {
    let mut table = [0xff; 256];
    let mut i = 0;
    while i < charset.len() {
        table[charset[i] as usize] = i as u8;
        i += 1;
    }
    table
}

pub(crate) const STANDARD_DECODE_TABLE: &[u8; 256] = &decode_table(STANDARD_CHARSET);
pub(crate) const URL_SAFE_DECODE_TABLE: &[u8; 256] = &decode_table(URL_SAFE_CHARSET);

#[inline]
pub fn encode<'s, 'd>(
    base64: &'_ Base64,
    src: &'s [u8],
    dst: OutBuf<'d>,
) -> Result<&'d mut [u8], Error> {
    unsafe {
        if src.is_empty() {
            return Ok(empty_slice_mut(dst.as_mut_ptr()));
        }

        let n = src.len();
        let m = Base64::encoded_length_unchecked(n, base64.padding);

        if dst.len() < m {
            return Err(ERROR);
        }

        let charset = match base64.kind {
            Base64Kind::Standard => STANDARD_CHARSET.as_ptr(),
            Base64Kind::UrlSafe => URL_SAFE_CHARSET.as_ptr(),
        };
        let padding = base64.padding;

        {
            let mut src = src.as_ptr();
            let mut dst = dst.as_mut_ptr();

            let dst_end = dst.add(n / 3 * 4);

            const UNROLL: usize = 4;
            if n / 3 * 3 >= (UNROLL * 6 + 2) {
                let src_end = src.add(n / 3 * 3 - (UNROLL * 6 + 2));
                while src <= src_end {
                    for _ in 0..UNROLL {
                        let x = u64::from_be_bytes(src.cast::<[u8; 8]>().read());
                        for i in 0..8 {
                            let y = read(charset, ((x >> (58 - i * 6)) & 0x3f) as usize);
                            write(dst, i, y)
                        }
                        src = src.add(6);
                        dst = dst.add(8);
                    }
                }
            }

            while dst < dst_end {
                let x = u32::from_be_bytes([0, read(src, 0), read(src, 1), read(src, 2)]);
                for i in 0..4 {
                    let y = read(charset, ((x >> (18 - i * 6)) & 0x3f) as usize);
                    write(dst, i, y);
                }
                src = src.add(3);
                dst = dst.add(4);
            }

            encode_extra(n % 3, src, dst, charset, padding)
        }

        Ok(slice::from_raw_parts_mut(dst.as_mut_ptr(), m))
    }
}

pub(crate) unsafe fn encode_extra(
    extra: usize,
    src: *const u8,
    dst: *mut u8,
    charset: *const u8,
    padding: bool,
) {
    match extra {
        0 => {}
        1 => {
            let x = read(src, 0);
            let y1 = read(charset, (x >> 2) as usize);
            let y2 = read(charset, ((x << 6) >> 2) as usize);
            write(dst, 0, y1);
            write(dst, 1, y2);
            if padding {
                write(dst, 2, Base64::PAD);
                write(dst, 3, Base64::PAD);
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
                write(dst, 3, Base64::PAD);
            }
        }
        _ => core::hint::unreachable_unchecked(),
    }
}

#[inline]
pub fn decode<'s, 'd>(
    base64: &'_ Base64,
    src: &'s [u8],
    dst: OutBuf<'d>,
) -> Result<&'d mut [u8], Error> {
    unsafe {
        if src.is_empty() {
            return Ok(empty_slice_mut(dst.as_mut_ptr()));
        }

        let (n, m) = Base64::decoded_length_unchecked(src, base64.padding)?;

        if dst.len() < m {
            return Err(ERROR);
        }

        let src = src.as_ptr();
        let dst = dst.as_mut_ptr();
        decode_unchecked(base64, n, m, src, dst)?;

        Ok(slice::from_raw_parts_mut(dst, m))
    }
}

#[inline]
pub fn decode_inplace<'b>(base64: &'_ Base64, buf: &'b mut [u8]) -> Result<&'b mut [u8], Error> {
    unsafe {
        if buf.is_empty() {
            return Ok(empty_slice_mut(buf.as_mut_ptr()));
        }

        let (n, m) = Base64::decoded_length_unchecked(buf, base64.padding)?;

        let src = buf.as_ptr();
        let dst = buf.as_mut_ptr();
        decode_unchecked(base64, n, m, src, dst)?;

        Ok(slice::from_raw_parts_mut(dst, m))
    }
}

unsafe fn decode_unchecked(
    base64: &'_ Base64,
    n: usize,
    m: usize,
    mut src: *const u8,
    mut dst: *mut u8,
) -> Result<(), Error> {
    let table = match base64.kind {
        Base64Kind::Standard => STANDARD_DECODE_TABLE.as_ptr(),
        Base64Kind::UrlSafe => URL_SAFE_DECODE_TABLE.as_ptr(),
    };

    let src_end = src.add(n / 4 * 4);

    const UNROLL: usize = 4;
    if m >= (UNROLL * 6 + 2) {
        let end = dst.add(m - (UNROLL * 6 + 2));
        while dst <= end {
            for _ in 0..UNROLL {
                let mut x = u64::from_le_bytes(src.cast::<[u8; 8]>().read());
                let mut y: u64 = 0;
                let mut flag = 0;
                for i in 0..8 {
                    let bits = read(table, (x & 0xff) as usize);
                    flag |= bits;
                    x >>= 8;
                    y |= (bits as u64) << (58 - i * 6);
                }
                if flag == 0xff {
                    return Err(ERROR);
                }
                #[cfg(target_endian = "little")]
                {
                    y = y.swap_bytes();
                }
                dst.cast::<u64>().write_unaligned(y);

                src = src.add(8);
                dst = dst.add(6);
            }
        }
    }

    while src < src_end {
        let mut x = u32::from_le_bytes(src.cast::<[u8; 4]>().read());
        let mut y: u32 = 0;
        let mut flag = 0;
        for i in 0..4 {
            let bits = read(table, (x & 0xff) as usize);
            flag |= bits;
            x >>= 8;
            y |= (bits as u32) << (18 - i * 6);
        }
        if flag == 0xff {
            return Err(ERROR);
        }
        let y = y.to_be_bytes();
        write(dst, 0, y[1]);
        write(dst, 1, y[2]);
        write(dst, 2, y[3]);
        src = src.add(4);
        dst = dst.add(3);
    }

    decode_extra(n % 4, src, dst, table)?;

    Ok(())
}

pub(crate) unsafe fn decode_extra(
    extra: usize,
    src: *const u8,
    dst: *mut u8,
    table: *const u8,
) -> Result<(), Error> {
    match extra {
        0 => {}
        1 => core::hint::unreachable_unchecked(),
        2 => {
            let [x1, x2] = src.cast::<[u8; 2]>().read();
            let y1 = read(table, x1 as usize);
            let y2 = read(table, x2 as usize);
            if (y2 & 0x0f) != 0 {
                return Err(ERROR);
            }
            if (y1 | y2) == 0xff {
                return Err(ERROR);
            }
            write(dst, 0, (y1 << 2) | (y2 >> 4));
        }
        3 => {
            let [x1, x2, x3] = src.cast::<[u8; 3]>().read();
            let y1 = read(table, x1 as usize);
            let y2 = read(table, x2 as usize);
            let y3 = read(table, x3 as usize);
            if (y3 & 0x03) != 0 {
                return Err(ERROR);
            }
            if (y1 | y2 | y3) == 0xff {
                return Err(ERROR);
            }
            write(dst, 0, (y1 << 2) | (y2 >> 4));
            write(dst, 1, (y2 << 4) | (y3 >> 2));
        }
        _ => core::hint::unreachable_unchecked(),
    }
    Ok(())
}

#[test]
fn test() {
    crate::tests::test(encode, decode, decode_inplace);
}
