use crate::{Error, Kind};

use vsimd::base64::{STANDARD_ALSW_CHECK, URL_SAFE_ALSW_CHECK};
use vsimd::base64::{STANDARD_ALSW_DECODE, URL_SAFE_ALSW_DECODE};
use vsimd::base64::{STANDARD_CHARSET, URL_SAFE_CHARSET};

use vsimd::tools::{read, write};
use vsimd::SIMD256;

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
pub fn decoded_length(src: &[u8], padding: bool) -> Result<(usize, usize), Error> {
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

pub(crate) unsafe fn decode_fallback(
    mut src: *const u8,
    mut dst: *mut u8,
    mut n: usize,
    kind: Kind,
) -> Result<(), Error> {
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

pub(crate) unsafe fn decode_simd<S: SIMD256>(
    s: S,
    mut src: *const u8,
    mut dst: *mut u8,
    mut n: usize,
    kind: Kind,
) -> Result<(), Error> {
    let (check_lut, decode_lut) = match kind {
        Kind::Standard => (STANDARD_ALSW_CHECK, STANDARD_ALSW_DECODE),
        Kind::UrlSafe => (URL_SAFE_ALSW_CHECK, URL_SAFE_ALSW_DECODE),
    };

    // n*3/4 >= 24+4
    while n >= 38 {
        let x = s.v256_load_unaligned(src);
        let y = vsimd::base64::decode_ascii32(s, x, check_lut, decode_lut).map_err(|()| Error::new())?;

        let (y1, y2) = y.to_v128x2();
        s.v128_store_unaligned(dst, y1);
        s.v128_store_unaligned(dst.add(12), y2);

        src = src.add(32);
        dst = dst.add(24);
        n -= 32;
    }

    decode_fallback(src, dst, n, kind)
}
