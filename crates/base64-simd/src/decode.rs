use crate::error::{Error, ERROR};
use crate::{Base64, Base64Kind, STANDARD_CHARSET, URL_SAFE_CHARSET};

use simd_abstraction::tools::{read, write};

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
pub unsafe fn decode_raw_fallback(
    base64: &Base64,
    n: usize,
    m: usize,
    mut src: *const u8,
    mut dst: *mut u8,
) -> Result<(), Error> {
    let table: *const u8 = match base64.kind {
        Base64Kind::Standard => STANDARD_DECODE_TABLE.as_ptr(),
        Base64Kind::UrlSafe => URL_SAFE_DECODE_TABLE.as_ptr(),
    };

    let src_end = src.add(n / 4 * 4);

    const UNROLL: usize = 4;
    if m >= (UNROLL * 6 + 2) {
        let end = dst.add(m - (UNROLL * 6 + 2));
        while dst <= end {
            for _ in 0..UNROLL {
                let mut x = src.cast::<u64>().read_unaligned();
                #[cfg(target_endian = "big")]
                {
                    x = x.swap_bytes();
                }
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
        let mut x = src.cast::<u32>().read_unaligned();
        #[cfg(target_endian = "big")]
        {
            x = x.swap_bytes();
        }
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

#[inline(always)]
unsafe fn decode_extra(
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
