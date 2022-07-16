#![allow(clippy::missing_safety_doc)]

use crate::fallback::{decode_extra, encode_extra};
use crate::fallback::{STANDARD_CHARSET, URL_SAFE_CHARSET};
use crate::fallback::{STANDARD_DECODE_TABLE, URL_SAFE_DECODE_TABLE};
use crate::{Base64, Base64Kind, Error, OutBuf, ERROR};

use simd_abstraction::tools::{read, slice_mut, write};
use simd_abstraction::tools::{Bytes32, Load};
use simd_abstraction::traits::SIMD256;

use core::slice;

macro_rules! specialize_for {
    ($feature:literal, $ty: ty) => {
        use crate::{Base64, Error, OutBuf};
        use simd_abstraction::traits::InstructionSet;

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn encode<'s, 'd>(
            base64: &'_ Base64,
            src: &'s [u8],
            dst: OutBuf<'d>,
        ) -> Result<&'d mut [u8], Error> {
            let s = <$ty as InstructionSet>::new();
            crate::generic::encode(s, base64, src, dst)
        }

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn decode<'s, 'd>(
            base64: &'_ Base64,
            src: &'s [u8],
            dst: OutBuf<'d>,
        ) -> Result<&'d mut [u8], Error> {
            let s = <$ty as InstructionSet>::new();
            crate::generic::decode(s, base64, src, dst)
        }

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn decode_inplace<'b>(
            base64: &'_ Base64,
            buf: &'b mut [u8],
        ) -> Result<&'b mut [u8], Error> {
            let s = <$ty as InstructionSet>::new();
            crate::generic::decode_inplace(s, base64, buf)
        }
    };
}

pub unsafe trait SIMDExt: SIMD256 {
    fn base64_split_bits(self, a: Self::V256) -> Self::V256 {
        // a : {bbbbcccc|aaaaaabb|ccdddddd|bbbbcccc} x8 (1021)

        let m1 = self.u32x8_splat(u32::from_le_bytes([0x00, 0xfc, 0x00, 0x00]));
        let a1 = self.u16x16_shr::<10>(self.v256_and(a, m1));
        // a1: {00aaaaaa|000000000|00000000|00000000} x8

        let m2 = self.u32x8_splat(u32::from_le_bytes([0xf0, 0x03, 0x00, 0x00]));
        let a2 = self.u16x16_shl::<4>(self.v256_and(a, m2));
        // a2: {00000000|00bbbbbb|00000000|00000000} x8

        let m3 = self.u32x8_splat(u32::from_le_bytes([0x00, 0x00, 0xc0, 0x0f]));
        let a3 = self.u16x16_shr::<6>(self.v256_and(a, m3));
        // a3: {00000000|00000000|00cccccc|00000000} x8

        let m4 = self.u32x8_splat(u32::from_le_bytes([0x00, 0x00, 0x3f, 0x00]));
        let a4 = self.u16x16_shl::<8>(self.v256_and(a, m4));
        // a4: {00000000|00000000|00000000|00dddddd} x8

        self.v256_or(self.v256_or(a1, a2), self.v256_or(a3, a4))
        // {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8
    }

    fn base64_merge_bits(self, a: Self::V256) -> Self::V256 {
        // a : {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8

        let m1 = self.u32x8_splat(u32::from_le_bytes([0x3f, 0x00, 0x3f, 0x00]));
        let a1 = self.v256_and(a, m1);
        // a1: {00aaaaaa|00000000|00cccccc|00000000} x8

        let m2 = self.u32x8_splat(u32::from_le_bytes([0x00, 0x3f, 0x00, 0x3f]));
        let a2 = self.v256_and(a, m2);
        // a2: {00000000|00bbbbbb|00000000|00dddddd} x8

        let a3 = self.v256_or(self.u32x8_shl::<18>(a1), self.u32x8_shr::<10>(a1));
        // a3: {cc000000|0000cccc|aaaaaa00|00000000} x8

        let a4 = self.v256_or(self.u32x8_shl::<4>(a2), self.u32x8_shr::<24>(a2));
        // a4: {00dddddd|bbbb0000|000000bb|dddd0000}

        let mask = self.u32x8_splat(u32::from_le_bytes([0xff, 0xff, 0xff, 0x00]));
        self.v256_and(self.v256_or(a3, a4), mask)
        // {ccdddddd|bbbbcccc|aaaaaabb|00000000} x8
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

pub fn encode<'s, 'd, S: SIMDExt>(
    s: S,
    base64: &'_ Base64,
    src: &'s [u8],
    mut dst: OutBuf<'d>,
) -> Result<&'d mut [u8], Error> {
    unsafe {
        if src.is_empty() {
            return Ok(slice_mut(dst.as_mut_ptr(), 0));
        }
        let n = src.len();
        let m = Base64::encoded_length_unchecked(n, base64.padding);

        if dst.len() < m {
            return Err(ERROR);
        }

        let (charset, shift_lut) = match base64.kind {
            Base64Kind::Standard => (STANDARD_CHARSET.as_ptr(), STANDARD_ENCODING_SHIFT),
            Base64Kind::UrlSafe => (URL_SAFE_CHARSET.as_ptr(), URL_SAFE_ENCODING_SHIFT),
        };

        {
            let mut src = src.as_ptr();
            let mut dst = dst.as_mut_ptr();

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
                    let y = encode_chunk(s, x, shift_lut);
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

        Ok(slice::from_raw_parts_mut(dst.as_mut_ptr(), m))
    }
}

#[inline(always)]
unsafe fn encode_chunk<S: SIMDExt>(s: S, x: S::V256, shift_lut: S::V256) -> S::V256 {
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

pub fn decode<'s, 'd, S: SIMDExt>(
    s: S,
    base64: &'_ Base64,
    src: &'s [u8],
    mut dst: OutBuf<'d>,
) -> Result<&'d mut [u8], Error> {
    unsafe {
        if src.is_empty() {
            return Ok(slice_mut(dst.as_mut_ptr(), 0));
        }

        let (n, m) = Base64::decoded_length_unchecked(src, base64.padding)?;

        if dst.len() < m {
            return Err(ERROR);
        }

        let src = src.as_ptr();
        let dst = dst.as_mut_ptr();
        decode_unchecked(s, base64, n, m, src, dst)?;

        Ok(slice::from_raw_parts_mut(dst, m))
    }
}

pub fn decode_inplace<'b, S: SIMDExt>(
    s: S,
    base64: &'_ Base64,
    buf: &'b mut [u8],
) -> Result<&'b mut [u8], Error> {
    unsafe {
        if buf.is_empty() {
            return Ok(slice_mut(buf.as_mut_ptr(), 0));
        }

        let (n, m) = Base64::decoded_length_unchecked(buf, base64.padding)?;

        let src = buf.as_ptr();
        let dst = buf.as_mut_ptr();
        decode_unchecked(s, base64, n, m, src, dst)?;

        Ok(slice::from_raw_parts_mut(dst, m))
    }
}

#[inline(always)]
unsafe fn decode_unchecked<S: SIMDExt>(
    s: S,
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

    if m >= (24 + 4) {
        let end = dst.add(m - (24 + 4));
        let range_check = B64Range::new(s, base64);
        while dst <= end {
            let x = s.v256_load_unaligned(src);
            let y = decode_chunk(s, x, range_check)?;
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

impl<S: SIMD256> B64Range<S> {
    #[inline(always)]
    fn new(s: S, base64: &Base64) -> Self {
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
            Base64Kind::Standard => Self {
                lower_limit: s.load(&STANDARD_LIMITS.0),
                upper_limit: s.load(&STANDARD_LIMITS.1),
                decoding_shift: s.load(STANDARD_DECODING_SHIFT),
                b63: s.u8x32_splat(b'/'),
                b63_shift63: s.u8x32_splat(253), //  (63 - b'/') - (62 - b'+')
            },
            Base64Kind::UrlSafe => Self {
                lower_limit: s.load(&URL_SAFE_LIMITS.0),
                upper_limit: s.load(&URL_SAFE_LIMITS.1),
                decoding_shift: s.load(URL_SAFE_DECODING_SHIFT),
                b63: s.u8x32_splat(b'_'),
                b63_shift63: s.u8x32_splat(33), // (63 - b'_') - (15 - b'P')
            },
        }
    }
}

#[inline(always)]
unsafe fn decode_chunk<S: SIMDExt>(s: S, x: S::V256, r: B64Range<S>) -> Result<S::V256, Error> {
    // x: {{ascii}} x32

    let hi = s.v256_and(s.u16x16_shr::<4>(x), s.u8x32_splat(0x0f));
    let lower_limit = s.u8x16x2_swizzle(r.lower_limit, hi);
    let upper_limit = s.u8x16x2_swizzle(r.upper_limit, hi);

    let c1 = s.i8x32_cmp_lt(x, lower_limit);
    let c2 = s.i8x32_cmp_lt(upper_limit, x);
    let c3 = s.v256_or(c1, c2);
    let c4 = s.i8x32_cmp_eq(x, r.b63);
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

    const SHUFFLE: &Bytes32 = &Bytes32([
        0x02, 0x01, 0x00, 0x06, 0x05, 0x04, 0x0a, 0x09, //
        0x08, 0x0e, 0x0d, 0x0c, 0x80, 0x80, 0x80, 0x80, //
        0x02, 0x01, 0x00, 0x06, 0x05, 0x04, 0x0a, 0x09, //
        0x08, 0x0e, 0x0d, 0x0c, 0x80, 0x80, 0x80, 0x80, //
    ]);
    Ok(s.u8x16x2_swizzle(x4, s.load(SHUFFLE)))
    // {AAAB|BBCC|CDDD|????|EEEF|FFGG|GHHH|????}
}
