use crate::spec::SIMDExt;
use crate::{Base64, Base64Kind, STANDARD_CHARSET, URL_SAFE_CHARSET};

use simd_abstraction::scalar::Bytes32;
use simd_abstraction::tools::{read, write};
use simd_abstraction::traits::SimdLoad;

#[inline(always)]
pub const unsafe fn encoded_length_unchecked(n: usize, padding: bool) -> usize {
    let extra = n % 3;
    if extra == 0 {
        n / 3 * 4
    } else if padding {
        n / 3 * 4 + 4
    } else {
        n / 3 * 4 + extra + 1
    }
}

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

const fn encoding_shift(charset: &'static [u8; 64]) -> Bytes32 {
    let mut lut = [0x80; 32];
    let mut j = 0;
    while j < 32 {
        lut[j + 13] = b'A';
        lut[j] = b'a' - 26;
        let mut i = 1;
        while i <= 10 {
            lut[j + i] = b'0'.wrapping_sub(52);
            i += 1;
        }
        lut[j + 11] = charset[62].wrapping_sub(62);
        lut[j + 12] = charset[63].wrapping_sub(63);
        j += 16;
    }
    Bytes32(lut)
}

const STANDARD_ENCODING_SHIFT: &Bytes32 = &encoding_shift(STANDARD_CHARSET);
const URL_SAFE_ENCODING_SHIFT: &Bytes32 = &encoding_shift(URL_SAFE_CHARSET);

pub unsafe fn encode_raw_simd<S: SIMDExt>(s: S, base64: &Base64, src: &[u8], dst: *mut u8) {
    let (charset, shift_lut) = match base64.kind {
        Base64Kind::Standard => (STANDARD_CHARSET.as_ptr(), STANDARD_ENCODING_SHIFT),
        Base64Kind::UrlSafe => (URL_SAFE_CHARSET.as_ptr(), URL_SAFE_ENCODING_SHIFT),
    };

    let n: usize = src.len();
    let mut src: *const u8 = src.as_ptr();
    let mut dst: *mut u8 = dst;
    let src_end = src.add(n / 3 * 3);

    if n >= (28 + 6) {
        for _ in 0..2 {
            let x = u32::from_be_bytes([0, read(src, 0), read(src, 1), read(src, 2)]);
            for i in 0..4 {
                let y = read(charset, ((x >> (18 - i * 6)) & 0x3f) as usize);
                write(dst, i, y);
            }
            src = src.add(3);
            dst = dst.add(4);
        }

        let end = src.add(n - (28 + 6));
        let shift_lut = s.load(shift_lut);
        while src <= end {
            let x = s.v256_load_unaligned(src.sub(4));
            let y = encode_u8x32(s, x, shift_lut);
            s.v256_store_unaligned(dst, y);
            src = src.add(24);
            dst = dst.add(32);
        }
    }

    while src < src_end {
        let x = u32::from_be_bytes([0, read(src, 0), read(src, 1), read(src, 2)]);
        for i in 0..4 {
            let y = read(charset, ((x >> (18 - i * 6)) & 0x3f) as usize);
            write(dst, i, y);
        }
        src = src.add(3);
        dst = dst.add(4);
    }
    encode_extra(n % 3, src, dst, charset, base64.padding)
}

#[inline(always)]
unsafe fn encode_u8x32<S: SIMDExt>(s: S, x: S::V256, shift_lut: S::V256) -> S::V256 {
    // x: {????|AAAB|BBCC|CDDD|EEEF|FFGG|GHHH|????}

    const SHUFFLE: &Bytes32 = &Bytes32([
        0x05, 0x04, 0x06, 0x05, 0x08, 0x07, 0x09, 0x08, //
        0x0b, 0x0a, 0x0c, 0x0b, 0x0e, 0x0d, 0x0f, 0x0e, //
        0x01, 0x00, 0x02, 0x01, 0x04, 0x03, 0x05, 0x04, //
        0x07, 0x06, 0x08, 0x07, 0x0a, 0x09, 0x0b, 0x0a, //
    ]);

    let x0 = s.u8x16x2_swizzle(x, s.load(SHUFFLE));
    // x0: {bbbbcccc|aaaaaabb|ccdddddd|bbbbcccc} x8 (1021)

    let x1 = s.base64_split_bits(x0);
    // x1: {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8

    let x2 = s.u8x32_sub_sat(x1, s.u8x32_splat(51));
    // 0~25    => 0
    // 26~51   => 0
    // 52~61   => 1~10
    // 62      => 11
    // 63      => 12

    let x3 = s.i8x32_cmp_lt(x1, s.u8x32_splat(26));
    let x4 = s.v256_and(x3, s.u8x32_splat(13));
    let x5 = s.v256_or(x2, x4);
    // 0~25    => 0xff  => 13
    // 26~51   => 0     => 0
    // 52~61   => 0     => 1~10
    // 62      => 0     => 11
    // 63      => 0     => 12

    let shift = s.u8x16x2_swizzle(shift_lut, x5);
    s.u8x32_add(x1, shift)
    // {{ascii}} x32
}
