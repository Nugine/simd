use core::ptr::null_mut;

use crate::{Config, Error, Extra, Kind};

use vsimd::base64::{STANDARD_CHARSET, URL_SAFE_CHARSET};

use vsimd::tools::{read, slice_parts, write};

#[inline(always)]
pub(crate) const fn encoded_length_unchecked(len: usize, config: Config) -> usize {
    let extra = len % 3;
    if extra == 0 {
        len / 3 * 4
    } else if config.extra.padding() {
        len / 3 * 4 + 4
    } else {
        len / 3 * 4 + extra + 1
    }
}

#[inline(always)]
pub unsafe fn encode_bits24(src: *const u8, dst: *mut u8, charset: *const u8) {
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

pub(crate) unsafe fn encode(src: &[u8], mut dst: *mut u8, config: Config) {
    let kind = config.kind;
    let padding = config.extra.padding();

    let charset = match kind {
        Kind::Standard => STANDARD_CHARSET.as_ptr(),
        Kind::UrlSafe => URL_SAFE_CHARSET.as_ptr(),
    };

    let (mut src, mut len) = slice_parts(src);

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

const fn decode_table(charset: &'static [u8; 64]) -> [u8; 256] {
    let mut table = [0xff; 256];
    let mut i = 0;
    while i < charset.len() {
        table[charset[i] as usize] = i as u8;
        i += 1;
    }
    table
}

pub const STANDARD_DECODE_TABLE: &[u8; 256] = &decode_table(STANDARD_CHARSET);
pub const URL_SAFE_DECODE_TABLE: &[u8; 256] = &decode_table(URL_SAFE_CHARSET);

#[inline(always)]
pub(crate) fn decoded_length(src: &[u8], config: Config) -> Result<(usize, usize), Error> {
    if src.is_empty() {
        return Ok((0, 0));
    }

    let n = unsafe {
        let len = src.len();

        let count_pad = || {
            let last1 = *src.get_unchecked(len - 1);
            let last2 = *src.get_unchecked(len - 2);
            if last1 == b'=' {
                if last2 == b'=' {
                    2
                } else {
                    1
                }
            } else {
                0
            }
        };

        match config.extra {
            Extra::Pad => {
                ensure!(len % 4 == 0);
                len - count_pad()
            }
            Extra::NoPad => len,
            Extra::Forgiving => {
                if len % 4 == 0 {
                    len - count_pad()
                } else {
                    len
                }
            }
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
unsafe fn decode_ascii8<const WRITE: bool>(src: *const u8, dst: *mut u8, table: *const u8) -> Result<(), Error> {
    let mut x = u64::from_le_bytes(src.cast::<[u8; 8]>().read());
    let mut y: u64 = 0;
    let mut flag = 0;

    for i in 0..8 {
        let bits = read(table, (x & 0xff) as usize);
        flag |= bits;
        x >>= 8;

        if WRITE {
            y |= (bits as u64) << (58 - i * 6);
        }
    }
    ensure!(flag != 0xff);

    if WRITE {
        dst.cast::<u64>().write_unaligned(y.to_be());
    }
    Ok(())
}

#[inline(always)]
unsafe fn decode_ascii4<const WRITE: bool>(src: *const u8, dst: *mut u8, table: *const u8) -> Result<(), Error> {
    let mut x = u32::from_le_bytes(src.cast::<[u8; 4]>().read());
    let mut y: u32 = 0;
    let mut flag = 0;

    for i in 0..4 {
        let bits = read(table, (x & 0xff) as usize);
        flag |= bits;
        x >>= 8;

        if WRITE {
            y |= (bits as u32) << (18 - i * 6);
        }
    }
    ensure!(flag != 0xff);

    if WRITE {
        let y = y.to_be_bytes();
        write(dst, 0, y[1]);
        write(dst, 1, y[2]);
        write(dst, 2, y[3]);
    }
    Ok(())
}

#[inline(always)]
unsafe fn decode_extra<const WRITE: bool>(
    extra: usize,
    src: *const u8,
    dst: *mut u8,
    table: *const u8,
    forgiving: bool,
) -> Result<(), Error> {
    match extra {
        0 => {}
        1 => core::hint::unreachable_unchecked(),
        2 => {
            let [x1, x2] = src.cast::<[u8; 2]>().read();

            let y1 = read(table, x1 as usize);
            let y2 = read(table, x2 as usize);
            ensure!((y1 | y2) != 0xff && (forgiving || (y2 & 0x0f) == 0));

            if WRITE {
                write(dst, 0, (y1 << 2) | (y2 >> 4));
            }
        }
        3 => {
            let [x1, x2, x3] = src.cast::<[u8; 3]>().read();

            let y1 = read(table, x1 as usize);
            let y2 = read(table, x2 as usize);
            let y3 = read(table, x3 as usize);
            ensure!((y1 | y2 | y3) != 0xff && (forgiving || (y3 & 0x03) == 0));

            if WRITE {
                write(dst, 0, (y1 << 2) | (y2 >> 4));
                write(dst, 1, (y2 << 4) | (y3 >> 2));
            }
        }
        _ => core::hint::unreachable_unchecked(),
    }
    Ok(())
}

pub(crate) unsafe fn decode(mut src: *const u8, mut dst: *mut u8, mut n: usize, config: Config) -> Result<(), Error> {
    let kind = config.kind;
    let forgiving = config.extra.forgiving();

    let table = match kind {
        Kind::Standard => STANDARD_DECODE_TABLE.as_ptr(),
        Kind::UrlSafe => URL_SAFE_DECODE_TABLE.as_ptr(),
    };

    // n*3/4 >= 6+2
    while n >= 11 {
        decode_ascii8::<true>(src, dst, table)?;
        src = src.add(8);
        dst = dst.add(6);
        n -= 8;
    }

    while n >= 4 {
        decode_ascii4::<true>(src, dst, table)?;
        src = src.add(4);
        dst = dst.add(3);
        n -= 4;
    }

    decode_extra::<true>(n, src, dst, table, forgiving)
}

pub(crate) fn check(src: &[u8], config: Config) -> Result<(), Error> {
    let kind = config.kind;
    let forgiving = config.extra.forgiving();

    let (mut src, mut n) = (src.as_ptr(), src.len());

    let table = match kind {
        Kind::Standard => STANDARD_DECODE_TABLE.as_ptr(),
        Kind::UrlSafe => URL_SAFE_DECODE_TABLE.as_ptr(),
    };

    unsafe {
        // n*3/4 >= 6+2
        while n >= 11 {
            decode_ascii8::<false>(src, null_mut(), table)?;
            src = src.add(8);
            n -= 8;
        }

        while n >= 4 {
            decode_ascii4::<false>(src, null_mut(), table)?;
            src = src.add(4);
            n -= 4;
        }

        decode_extra::<false>(n, src, null_mut(), table, forgiving)
    }
}
