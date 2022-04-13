#![allow(unused_macros, clippy::missing_safety_doc)]

use crate::sa_hex;
use crate::{AsciiCase, Error, Hex, ERROR};

use core::mem::MaybeUninit;
use core::ptr;

use simd_abstraction::tools::{Bytes32, Load};
use simd_abstraction::traits::SIMD256;

macro_rules! specialize_for {
    ($feature:literal, $ty: ty) => {
        use crate::{AsciiCase, Error, Hex};
        use simd_abstraction::traits::InstructionSet;

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn parse(src: &[u8]) -> Result<[u8; 16], Error> {
            let s = <$ty as InstructionSet>::new_unchecked();
            crate::generic::parse(s, src)
        }

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn parse_simple(src: &[u8]) -> Result<[u8; 16], Error> {
            let s = <$ty as InstructionSet>::new_unchecked();
            crate::generic::parse_simple(s, src)
        }

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn parse_hyphenated(src: &[u8]) -> Result<[u8; 16], Error> {
            let s = <$ty as InstructionSet>::new_unchecked();
            crate::generic::parse_hyphenated(s, src)
        }

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn format_simple(src: &[u8; 16], case: AsciiCase) -> Hex<32> {
            let s = <$ty as InstructionSet>::new_unchecked();
            crate::generic::format_simple(s, src, case)
        }

        #[inline]
        #[target_feature(enable = $feature)]
        pub unsafe fn format_hyphenated(src: &[u8; 16], case: AsciiCase) -> Hex<36> {
            let s = <$ty as InstructionSet>::new_unchecked();
            crate::generic::format_hyphenated(s, src, case)
        }
    };
}

pub unsafe trait SIMDExt: SIMD256 {
    fn i16x16_set_lane7(self, a: Self::V256, x: i16) -> Self::V256;
    fn i32x8_set_lane7(self, a: Self::V256, x: i32) -> Self::V256;
    fn i32x4_get_lane3(self, a: Self::V128) -> i32;
    fn i16x8_get_lane7(self, a: Self::V128) -> i16;
}

pub(crate) fn parse<S: SIMDExt>(s: S, mut src: &[u8]) -> Result<[u8; 16], Error> {
    fn judge_other(src: &[u8]) -> Result<&[u8], Error> {
        match src.len() {
            // Microsoft GUID
            38 => {
                if src[0] == b'{' && src[37] == b'}' {
                    Ok(&src[1..37])
                } else {
                    Err(ERROR)
                }
            }
            // URN prefixed UUID
            45 => match src.strip_prefix(b"urn:uuid:") {
                Some(s) => Ok(s),
                None => Err(ERROR),
            },
            _ => Err(ERROR),
        }
    }

    let n = src.len();
    if n == 32 {
        return parse_simple(s, src);
    }
    if n != 36 {
        src = judge_other(src)?;
    }
    parse_hyphenated(s, src)
}

pub(crate) fn parse_simple<S: SIMD256>(s: S, src: &[u8]) -> Result<[u8; 16], Error> {
    if src.len() != 32 {
        return Err(ERROR);
    }
    let a = unsafe { s.v256_load_unaligned(src.as_ptr()) };
    match sa_hex::decode_u8x32(s, a) {
        Ok(ans) => Ok(s.v128_to_bytes(ans)),
        Err(()) => Err(ERROR),
    }
}

pub(crate) fn parse_hyphenated<S: SIMDExt>(s: S, src: &[u8]) -> Result<[u8; 16], Error> {
    if src.len() != 36 {
        return Err(ERROR);
    }

    let x = unsafe {
        [
            *src.get_unchecked(8),
            *src.get_unchecked(13),
            *src.get_unchecked(18),
            *src.get_unchecked(23),
        ]
    };
    if !matches!(x, [b'-', b'-', b'-', b'-']) {
        return Err(ERROR);
    }

    const SWIZZLE: &Bytes32 = &Bytes32([
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, //
        0x09, 0x0a, 0x0b, 0x0c, 0x0e, 0x0f, 0x80, 0x80, //
        0x03, 0x04, 0x05, 0x06, 0x08, 0x09, 0x0a, 0x0b, //
        0x0c, 0x0d, 0x0e, 0x0f, 0x80, 0x80, 0x80, 0x80, //
    ]);

    let base: *const u8 = src.as_ptr();
    unsafe {
        let a0 = s.v256_load_unaligned(base.cast());
        let a1 = s.u8x16x2_swizzle(a0, s.load(SWIZZLE));
        let a2 = s.i16x16_set_lane7(a1, base.add(16).cast::<i16>().read_unaligned());
        let a3 = s.i32x8_set_lane7(a2, base.add(32).cast::<i32>().read_unaligned());
        match sa_hex::decode_u8x32(s, a3) {
            Ok(ans) => Ok(s.v128_to_bytes(ans)),
            Err(()) => Err(ERROR),
        }
    }
}

const fn char_lut(case: AsciiCase) -> &'static Bytes32 {
    match case {
        AsciiCase::Lower => sa_hex::ENCODE_LOWER_LUT,
        AsciiCase::Upper => sa_hex::ENCODE_UPPER_LUT,
    }
}

pub(crate) fn format_simple<S: SIMD256>(s: S, src: &[u8; 16], case: AsciiCase) -> Hex<32> {
    unsafe {
        let lut = s.load(char_lut(case));
        let a = s.v128_load_unaligned(src.as_ptr());
        let buf = s.v256_to_bytes(sa_hex::encode_u8x16(s, a, lut));
        Hex::new_unchecked(buf)
    }
}

pub(crate) fn format_hyphenated<S: SIMDExt>(s: S, src: &[u8; 16], case: AsciiCase) -> Hex<36> {
    const SWIZZLE: &Bytes32 = &Bytes32([
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, //
        0x80, 0x08, 0x09, 0x0a, 0x0b, 0x80, 0x0c, 0x0d, //
        0x80, 0x80, 0x80, 0x00, 0x01, 0x02, 0x03, 0x80, //
        0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, //
    ]);

    const DASH: &Bytes32 = &Bytes32([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x2d, 0x00, 0x00, 0x00, 0x00, 0x2d, 0x00, 0x00, //
        0x00, 0x00, 0x2d, 0x00, 0x00, 0x00, 0x00, 0x2d, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);

    let mut buf: MaybeUninit<[u8; 36]> = MaybeUninit::uninit();
    let dst = buf.as_mut_ptr().cast::<u8>();

    let a = unsafe {
        let lut = s.load(char_lut(case));
        sa_hex::encode_u8x16(s, s.v128_load_unaligned(src.as_ptr()), lut)
    };

    {
        let a1 = s.u8x16x2_swizzle(a, s.load(SWIZZLE));
        let a2 = s.v256_or(a1, s.load(DASH));
        unsafe { s.v256_store_unaligned(dst, a2) };
    }

    unsafe {
        let bytes_28_31 = s.i32x4_get_lane3(s.v256_get_high(a)) as u32;
        let bytes_14_15 = s.i16x8_get_lane7(s.v256_get_low(a)) as u16;
        ptr::write_unaligned(dst.add(16).cast(), bytes_14_15);
        ptr::write_unaligned(dst.add(32).cast(), bytes_28_31);
        Hex::new_unchecked(buf.assume_init())
    }
}
