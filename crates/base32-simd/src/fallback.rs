use crate::Error;

use vsimd::base32::Kind;
use vsimd::base32::{BASE32HEX_CHARSET, BASE32_CHARSET};
use vsimd::tools::{read, slice_parts, write};

#[inline(always)]
unsafe fn check_extra(src: *const u8, extra: usize, table: *const u8) -> Result<(), Error> {
    match extra {
        0 => {}
        2 => {
            let (u10, flag) = decode_bits::<2>(src, table);
            ensure!(flag != 0xff && u10 & 0b11 == 0);
        }
        4 => {
            let (u20, flag) = decode_bits::<4>(src, table);
            ensure!(flag != 0xff && u20 & 0b1111 == 0);
        }
        5 => {
            let (u25, flag) = decode_bits::<5>(src, table);
            ensure!(flag != 0xff && u25 & 0b1 == 0);
        }
        7 => {
            let (u35, flag) = decode_bits::<7>(src, table);
            ensure!(flag != 0xff && u35 & 0b111 == 0);
        }
        _ => core::hint::unreachable_unchecked(),
    }
    Ok(())
}

#[inline(always)]
pub fn check(src: &[u8], kind: Kind) -> Result<(), Error> {
    let table = match kind {
        Kind::Base32 => BASE32_TABLE.as_ptr(),
        Kind::Base32Hex => BASE32HEX_TABLE.as_ptr(),
    };

    unsafe {
        let (mut src, mut len) = slice_parts(src);

        let end = src.add(len / 8 * 8);
        while src < end {
            let (_, flag) = decode_bits::<8>(src, table);
            ensure!(flag != 0xff);
            src = src.add(8);
        }
        len %= 8;

        check_extra(src, len, table)
    }
}

pub const fn encoded_length_unchecked(len: usize, padding: bool) -> usize {
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

#[inline(always)]
pub unsafe fn encode_bits<const N: usize>(dst: *mut u8, charset: *const u8, x: u64) {
    debug_assert!(matches!(N, 2 | 4 | 5 | 7 | 8));

    {
        let shift = (N - 1) * 5;
        write(dst, 0, read(charset, (x >> shift) as usize));
    }
    let mut i = 1;
    while i < N {
        let shift = (N - 1 - i) * 5;
        write(dst, i, read(charset, ((x >> shift) & 0x1f) as usize));
        i += 1;
    }
}

#[inline(always)]
pub unsafe fn read_be_bytes<const N: usize>(src: *const u8) -> u64 {
    debug_assert!(matches!(N, 1 | 2 | 3 | 4 | 5));

    #[cfg(not(target_arch = "wasm32"))]
    {
        if N == 3 {
            let x1: u8 = read(src, 0);
            let x2: u16 = src.add(1).cast::<u16>().read_unaligned().to_be();
            return ((x1 as u64) << 16) | (x2 as u64);
        }
        if N == 5 {
            let x1: u8 = read(src, 0);
            let x2: u32 = src.add(1).cast::<u32>().read_unaligned().to_be();
            return ((x1 as u64) << 32) | (x2 as u64);
        }
    }

    let mut ans = 0;
    let mut i = 0;
    while i < N {
        let shift = (N - 1 - i) * 8;
        ans |= (read(src, i) as u64) << shift;
        i += 1;
    }
    ans
}

#[inline(always)]
unsafe fn encode_extra(src: *const u8, extra: usize, dst: *mut u8, charset: *const u8, padding: bool) {
    match extra {
        0 => {}
        1 => {
            let u10 = read_be_bytes::<1>(src) << 2;
            encode_bits::<2>(dst, charset, u10);
            if padding {
                let mut i = 2;
                while i < 8 {
                    write(dst, i, b'=');
                    i += 1;
                }
            }
        }
        2 => {
            let u20 = read_be_bytes::<2>(src) << 4;
            encode_bits::<4>(dst, charset, u20);
            if padding {
                let mut i = 4;
                while i < 8 {
                    write(dst, i, b'=');
                    i += 1;
                }
            }
        }
        3 => {
            let u25 = read_be_bytes::<3>(src) << 1;
            encode_bits::<5>(dst, charset, u25);
            if padding {
                let mut i = 5;
                while i < 8 {
                    write(dst, i, b'=');
                    i += 1;
                }
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

#[inline(always)]
pub unsafe fn encode(src: &[u8], mut dst: *mut u8, kind: Kind, padding: bool) {
    let charset: *const u8 = match kind {
        Kind::Base32 => BASE32_CHARSET.as_ptr(),
        Kind::Base32Hex => BASE32HEX_CHARSET.as_ptr(),
    };

    let (mut src, mut len) = slice_parts(src);

    let end = src.add(len / 5 * 5);
    while src < end {
        let u40 = read_be_bytes::<5>(src);
        encode_bits::<8>(dst, charset, u40);
        src = src.add(5);
        dst = dst.add(8);
    }
    len %= 5;

    encode_extra(src, len, dst, charset, padding);
}

#[inline]
const fn decoding_table(charset: &[u8; 32]) -> [u8; 256] {
    let mut table = [0xff; 256];
    let mut i = 0;
    while i < 32 {
        table[charset[i] as usize] = i as u8;
        i += 1;
    }
    table
}

const BASE32_TABLE: &[u8; 256] = &decoding_table(BASE32_CHARSET);
const BASE32HEX_TABLE: &[u8; 256] = &decoding_table(BASE32HEX_CHARSET);

#[inline]
pub fn decoded_length(data: &[u8], padding: bool) -> Result<(usize, usize), Error> {
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

#[inline(always)]
unsafe fn decode_bits<const N: usize>(src: *const u8, table: *const u8) -> (u64, u8) {
    debug_assert!(matches!(N, 2 | 4 | 5 | 7 | 8));
    let mut ans: u64 = 0;
    let mut flag = 0;
    let mut i = 0;
    while i < N {
        let bits = read(table, read(src, i) as usize);
        flag |= bits;
        ans = (ans << 5) | u64::from(bits);
        i += 1;
    }
    (ans, flag)
}

#[inline(always)]
unsafe fn write_be_bytes<const N: usize>(dst: *mut u8, x: u64) {
    debug_assert!(matches!(N, 1 | 2 | 3 | 4 | 5));

    #[cfg(not(target_arch = "wasm32"))]
    {
        if N == 3 {
            let x1 = (x >> 16) as u8;
            let x2 = (x as u16).to_be();
            dst.write(x1);
            dst.add(1).cast::<u16>().write_unaligned(x2);
            return;
        }
        if N == 5 {
            let x1 = (x >> 32) as u8;
            let x2 = (x as u32).to_be();
            dst.write(x1);
            dst.add(1).cast::<u32>().write_unaligned(x2);
            return;
        }
    }

    let mut i = 0;
    while i < N {
        let shift = (N - 1 - i) * 8;
        write(dst, i, (x >> shift) as u8);
        i += 1;
    }
}

#[inline(always)]
unsafe fn decode_extra(src: *const u8, extra: usize, dst: *mut u8, table: *const u8) -> Result<(), Error> {
    match extra {
        0 => {}
        2 => {
            let (u10, flag) = decode_bits::<2>(src, table);
            ensure!(flag != 0xff && u10 & 0b11 == 0);
            write_be_bytes::<1>(dst, u10 >> 2);
        }
        4 => {
            let (u20, flag) = decode_bits::<4>(src, table);
            ensure!(flag != 0xff && u20 & 0b1111 == 0);
            write_be_bytes::<2>(dst, u20 >> 4);
        }
        5 => {
            let (u25, flag) = decode_bits::<5>(src, table);
            ensure!(flag != 0xff && u25 & 0b1 == 0);
            write_be_bytes::<3>(dst, u25 >> 1);
        }
        7 => {
            let (u35, flag) = decode_bits::<7>(src, table);
            ensure!(flag != 0xff && u35 & 0b111 == 0);
            write_be_bytes::<4>(dst, u35 >> 3);
        }
        _ => core::hint::unreachable_unchecked(),
    }
    Ok(())
}

#[inline(always)]
pub unsafe fn decode(mut src: *const u8, mut n: usize, mut dst: *mut u8, kind: Kind) -> Result<(), Error> {
    let table = match kind {
        Kind::Base32 => BASE32_TABLE.as_ptr(),
        Kind::Base32Hex => BASE32HEX_TABLE.as_ptr(),
    };

    let end = src.add(n / 8 * 8);
    while src < end {
        let (u40, flag) = decode_bits::<8>(src, table);
        ensure!(flag != 0xff);
        write_be_bytes::<5>(dst, u40);
        src = src.add(8);
        dst = dst.add(5);
    }
    n %= 8;

    decode_extra(src, n, dst, table)
}
