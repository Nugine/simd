#![allow(clippy::missing_safety_doc)]

use crate::tools::{read, write};

#[inline(always)]
pub unsafe fn encode_bits<const N: usize>(dst: *mut u8, charset: *const u8, x: u64) {
    debug_assert!(matches!(N, 2 | 4 | 5 | 7 | 8));

    {
        let shift = (N - 1) * 5;
        write(dst, 0, read(charset, (x >> shift) as usize));
    }
    for i in 1..N {
        let shift = (N - 1 - i) * 5;
        write(dst, i, read(charset, ((x >> shift) & 0x1f) as usize));
    }
}

#[inline(always)]
pub unsafe fn decode_bits<const N: usize>(src: *const u8, table: *const u8) -> (u64, u8) {
    debug_assert!(matches!(N, 2 | 4 | 5 | 7 | 8));
    let mut ans: u64 = 0;
    let mut flag = 0;
    for i in 0..N {
        let bits = read(table, read(src, i) as usize);
        flag |= bits;
        ans = (ans << 5) | u64::from(bits);
    }
    (ans, flag)
}

#[inline(always)]
pub unsafe fn read_be_bytes<const N: usize>(src: *const u8) -> u64 {
    debug_assert!(matches!(N, 1 | 2 | 3 | 4 | 5));

    #[cfg(not(target_arch = "wasm32"))]
    {
        if N == 3 {
            let x1: u64 = read(src, 0).into();
            let x2: u64 = src.add(1).cast::<u16>().read_unaligned().to_be().into();
            return (x1 << 16) | x2;
        }
        if N == 5 {
            let x1: u64 = read(src, 0).into();
            let x2: u64 = src.add(1).cast::<u64>().read_unaligned().to_be();
            return (x1 << 32) | x2;
        }
    }

    let mut ans = 0;
    for i in 0..N {
        let shift = (N - 1 - i) * 8;
        ans |= u64::from(read(src, i)) << shift;
    }
    ans
}

#[inline(always)]
pub unsafe fn write_be_bytes<const N: usize>(dst: *mut u8, x: u64) {
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

    for i in 0..N {
        let shift = (N - 1 - i) * 8;
        write(dst, i, (x >> shift) as u8);
    }
}
