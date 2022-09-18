use crate::decode::{STANDARD_DECODE_TABLE, URL_SAFE_DECODE_TABLE};
use crate::{Error, Kind};

use vsimd::base64::{STANDARD_ALSW_CHECK, URL_SAFE_ALSW_CHECK};
use vsimd::tools::{read, slice};
use vsimd::SIMD256;

#[inline(always)]
unsafe fn check_ascii8(src: *const u8, table: *const u8) -> Result<(), Error> {
    let mut x = u64::from_le_bytes(src.cast::<[u8; 8]>().read());
    let mut flag = 0;
    for _ in 0..8 {
        let bits = read(table, (x & 0xff) as usize);
        flag |= bits;
        x >>= 8;
    }
    ensure!(flag != 0xff);
    Ok(())
}

#[inline(always)]
unsafe fn check_ascii4(src: *const u8, table: *const u8) -> Result<(), Error> {
    let mut x = u32::from_le_bytes(src.cast::<[u8; 4]>().read());
    let mut flag = 0;
    for _ in 0..4 {
        let bits = read(table, (x & 0xff) as usize);
        flag |= bits;
        x >>= 8;
    }
    ensure!(flag != 0xff);
    Ok(())
}

#[inline(always)]
unsafe fn check_extra(extra: usize, src: *const u8, table: *const u8) -> Result<(), Error> {
    match extra {
        0 => {}
        1 => core::hint::unreachable_unchecked(),
        2 => {
            let [x1, x2] = src.cast::<[u8; 2]>().read();
            let y1 = read(table, x1 as usize);
            let y2 = read(table, x2 as usize);
            ensure!(y2 & 0x0f == 0 && (y1 | y2) != 0xff);
        }
        3 => {
            let [x1, x2, x3] = src.cast::<[u8; 3]>().read();
            let y1 = read(table, x1 as usize);
            let y2 = read(table, x2 as usize);
            let y3 = read(table, x3 as usize);
            ensure!(y3 & 0x03 == 0 && (y1 | y2 | y3) != 0xff);
        }
        _ => core::hint::unreachable_unchecked(),
    }
    Ok(())
}

pub(crate) fn check_fallback(src: &[u8], kind: Kind) -> Result<(), Error> {
    let (mut src, mut n) = (src.as_ptr(), src.len());

    let table = match kind {
        Kind::Standard => STANDARD_DECODE_TABLE.as_ptr(),
        Kind::UrlSafe => URL_SAFE_DECODE_TABLE.as_ptr(),
    };

    unsafe {
        // n*3/4 >= 6+2
        while n >= 11 {
            check_ascii8(src, table)?;
            src = src.add(8);
            n -= 8;
        }

        while n >= 4 {
            check_ascii4(src, table)?;
            src = src.add(4);
            n -= 4;
        }

        check_extra(n, src, table)
    }
}

pub(crate) fn check_simd<S: SIMD256>(s: S, src: &[u8], kind: Kind) -> Result<(), Error> {
    let (mut src, mut n) = (src.as_ptr(), src.len());

    let check_lut = match kind {
        Kind::Standard => STANDARD_ALSW_CHECK,
        Kind::UrlSafe => URL_SAFE_ALSW_CHECK,
    };

    unsafe {
        // n*3/4 >= 24+4
        while n >= 38 {
            let x = s.v256_load_unaligned(src);
            let is_valid = vsimd::base64::check_ascii32(s, x, check_lut);
            ensure!(is_valid);
            src = src.add(32);
            n -= 32;
        }

        check_fallback(slice(src, n), kind)
    }
}
