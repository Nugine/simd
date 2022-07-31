use crate::error::{Error, ERROR};
use crate::spec::SIMDExt;
use crate::{Base64, Base64Kind, STANDARD_CHARSET, URL_SAFE_CHARSET};

use simd_abstraction::isa::{SimdLoad, SIMD256};
use simd_abstraction::scalar::Bytes32;
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
pub fn decoded_length(src: &[u8], padding: bool) -> Result<(usize, usize), Error> {
    if src.is_empty() {
        return Ok((0, 0));
    }

    let n = unsafe {
        let len = src.len();
        if padding {
            if len % 4 != 0 {
                return Err(ERROR);
            }
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
        1 => return Err(ERROR),
        2 => n / 4 * 3 + 1,
        3 => n / 4 * 3 + 2,
        _ => unsafe { core::hint::unreachable_unchecked() },
    };

    Ok((n, m))
}

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
unsafe fn decode_extra(extra: usize, src: *const u8, dst: *mut u8, table: *const u8) -> Result<(), Error> {
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

pub unsafe fn decode_raw_simd<S: SIMDExt>(
    s: S,
    base64: &Base64,
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

    if m >= (24 + 4) {
        let end = dst.add(m - (24 + 4));
        let range_check = b64_range(s, base64);
        while dst <= end {
            let x = s.v256_load_unaligned(src);
            let y = decode_u8x32(s, x, range_check)?;
            let (y1, y2) = s.v256_to_v128x2(y);
            s.v128_store_unaligned(dst, y1);
            s.v128_store_unaligned(dst.add(12), y2);
            src = src.add(32);
            dst = dst.add(24);
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

#[derive(Clone, Copy)]
struct B64Range<S: SIMD256> {
    lower_limit: S::V256,
    upper_limit: S::V256,
    decoding_shift: S::V256,
    b63: S::V256,
    b63_shift63: S::V256,
}

#[inline(always)]
fn b64_range<S: SIMD256>(s: S, base64: &Base64) -> B64Range<S> {
    const fn build_limits(b62: u8) -> (Bytes32, Bytes32) {
        let mut low: [u8; 32] = [0x01; 32];
        let mut high: [u8; 32] = [0x00; 32];
        let mut j = 0;
        while j < 32 {
            low[j + 4] = b'A';
            high[j + 4] = b'O';
            low[j + 5] = b'P';
            high[j + 5] = b'Z';
            low[j + 6] = b'a';
            high[j + 6] = b'o';
            low[j + 7] = b'p';
            high[j + 7] = b'z';
            low[j + 3] = b'0';
            high[j + 3] = b'9';
            low[j + (b62 >> 4) as usize] = b62;
            high[j + (b62 >> 4) as usize] = b62;
            j += 16;
        }
        (Bytes32(low), Bytes32(high))
    }

    const fn decoding_shift(b62: u8) -> Bytes32 {
        let mut lut = [0x00; 32];
        let mut j = 0;
        while j < 32 {
            lut[j + 4] = 191; // 0 - b'A'
            lut[j + 5] = 191; // 15 - b'P'
            lut[j + 6] = 185; // 26 - b'a'
            lut[j + 7] = 185; // 41 - b'p'
            lut[j + 3] = 4; // 52 - b'0'
            lut[j + (b62 >> 4) as usize] = 62_u8.wrapping_sub(b62);
            j += 16;
        }
        Bytes32(lut)
    }

    const STANDARD_LIMITS: (Bytes32, Bytes32) = build_limits(b'+');
    const URL_SAFE_LIMITS: (Bytes32, Bytes32) = build_limits(b'-');

    const STANDARD_DECODING_SHIFT: &Bytes32 = &decoding_shift(b'+');
    const URL_SAFE_DECODING_SHIFT: &Bytes32 = &decoding_shift(b'-');

    match base64.kind {
        Base64Kind::Standard => B64Range {
            lower_limit: s.load(&STANDARD_LIMITS.0),
            upper_limit: s.load(&STANDARD_LIMITS.1),
            decoding_shift: s.load(STANDARD_DECODING_SHIFT),
            b63: s.u8x32_splat(b'/'),
            b63_shift63: s.u8x32_splat(253), //  (63 - b'/') - (62 - b'+')
        },
        Base64Kind::UrlSafe => B64Range {
            lower_limit: s.load(&URL_SAFE_LIMITS.0),
            upper_limit: s.load(&URL_SAFE_LIMITS.1),
            decoding_shift: s.load(URL_SAFE_DECODING_SHIFT),
            b63: s.u8x32_splat(b'_'),
            b63_shift63: s.u8x32_splat(33), // (63 - b'_') - (15 - b'P')
        },
    }
}

#[inline(always)]
unsafe fn decode_u8x32<S: SIMDExt>(s: S, x: S::V256, r: B64Range<S>) -> Result<S::V256, Error> {
    // x: {{ascii}} x32

    let hi = s.v256_and(s.u16x16_shr::<4>(x), s.u8x32_splat(0x0f));
    let lower_limit = s.u8x16x2_swizzle(r.lower_limit, hi);
    let upper_limit = s.u8x16x2_swizzle(r.upper_limit, hi);

    let c1 = s.i8x32_lt(x, lower_limit);
    let c2 = s.i8x32_lt(upper_limit, x);
    let c3 = s.v256_or(c1, c2);
    let c4 = s.i8x32_eq(x, r.b63);
    let c5 = s.v256_andnot(c3, c4);

    if !s.v256_all_zero(c5) {
        return Err(ERROR);
    }

    let shift = s.u8x16x2_swizzle(r.decoding_shift, hi);
    let x1 = s.u8x32_add(x, shift);
    let x2 = s.v256_and(c4, r.b63_shift63);
    let x3 = s.u8x32_add(x1, x2);
    // x3: {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8

    let x4 = s.base64_merge_bits(x3);
    // {ccdddddd|bbbbcccc|aaaaaabb|00000000} x8

    const SHUFFLE: &Bytes32 = &Bytes32::double([
        0x02, 0x01, 0x00, 0x06, 0x05, 0x04, 0x0a, 0x09, //
        0x08, 0x0e, 0x0d, 0x0c, 0x80, 0x80, 0x80, 0x80, //
    ]);
    Ok(s.u8x16x2_swizzle(x4, s.load(SHUFFLE)))
    // {AAAB|BBCC|CDDD|????|EEEF|FFGG|GHHH|????}
}
