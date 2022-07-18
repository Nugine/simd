use crate::Base64;

use simd_abstraction::tools::{read, write};

pub unsafe fn encode_raw_fallback(base64: &Base64, src: &[u8], dst: *mut u8) {
    let charset: *const u8 = base64.charset().as_ptr();
    let padding = base64.padding;

    let n: usize = src.len();
    let mut src: *const u8 = src.as_ptr();
    let mut dst: *mut u8 = dst;
    let dst_end: *mut u8 = dst.add(n / 3 * 4);

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

#[inline(always)]
unsafe fn encode_extra(
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
