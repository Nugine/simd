use core::fmt;

use crate::tools::{Bytes32, Load};
use crate::traits::SIMD256;

#[inline]
pub fn check_u8x32<S: SIMD256>(s: S, a: S::V256) -> bool {
    let a1 = s.u8x32_add(a, s.u8x32_splat(0x50));
    let a2 = s.v256_and(a1, s.u8x32_splat(0xdf));
    let a3 = s.u8x32_sub(a2, s.u8x32_splat(0x11));
    let a4 = s.i8x32_cmp_lt(a1, s.i8x32_splat(-118));
    let a5 = s.i8x32_cmp_lt(a3, s.i8x32_splat(-122));
    let a6 = s.v256_or(a4, a5);
    !s.v256_all_zero(a6)
}

fn check_u8x32_hilo<S: SIMD256>(s: S, hi: S::V256, lo: S::V256) -> bool {
    const HI_LUT: &Bytes32 = &Bytes32([
        0x00, 0x00, 0x00, 0x0f, 0xf0, 0x00, 0xf0, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x0f, 0xf0, 0x00, 0xf0, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);
    const LI_LUT: &Bytes32 = &Bytes32([
        0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, //
        0x0f, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, //
        0x0f, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);

    let hi_check = s.u8x16x2_swizzle(s.load(HI_LUT), hi);
    let lo_check = s.u8x16x2_swizzle(s.load(LI_LUT), lo);
    let check = s.v256_and(hi_check, lo_check);

    !s.u8x32_any_zero(check)
}

#[allow(clippy::result_unit_err)]
#[inline]
pub fn decode_u8x32<S: SIMD256>(s: S, a: S::V256) -> Result<S::V128, ()> {
    let hi = s.u16x16_shr::<4>(s.v256_and(a, s.u8x32_splat(0xf0)));
    let lo = s.v256_and(a, s.u8x32_splat(0x0f));

    if !check_u8x32_hilo(s, hi, lo) {
        return Err(());
    }

    const OFFSET_LUT: &Bytes32 = &Bytes32([
        0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x09, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x09, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    ]);

    const SHUFFLE: &Bytes32 = &Bytes32([
        0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, //
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
        0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, //
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, //
    ]);

    let offset = s.u8x16x2_swizzle(s.load(OFFSET_LUT), hi);

    let a1 = s.u8x32_add(lo, offset);
    let a2 = s.u16x16_shl::<4>(a1);
    let a3 = s.u16x16_shr::<12>(a2);
    let a4 = s.v256_or(a2, a3);
    let a5 = s.u8x16x2_swizzle(a4, s.load(SHUFFLE));
    let a6 = s.u64x4_unzip_low(a5);

    Ok(a6)
}

pub const ENCODE_UPPER_LUT: &Bytes32 = &Bytes32(*b"0123456789ABCDEF0123456789ABCDEF");
pub const ENCODE_LOWER_LUT: &Bytes32 = &Bytes32(*b"0123456789abcdef0123456789abcdef");

#[inline]
pub fn encode_u8x16<S: SIMD256>(s: S, a: S::V128, lut: S::V256) -> S::V256 {
    let a0 = s.u16x16_from_u8x16(a);
    let a1 = s.u16x16_shl::<8>(a0);
    let a2 = s.u16x16_shr::<4>(a0);
    let a3 = s.v256_and(s.v256_or(a1, a2), s.u8x32_splat(0x0f));
    s.u8x16x2_swizzle(lut, a3)
}

#[inline(always)]
pub const fn unhex(x: u8) -> u8 {
    const UNHEX_TABLE: &[u8; 256] = &{
        let mut buf = [0; 256];
        let mut i: usize = 0;
        while i < 256 {
            let x = i as u8;
            buf[i] = match x {
                b'0'..=b'9' => x - b'0',
                b'a'..=b'f' => x - b'a' + 10,
                b'A'..=b'F' => x - b'A' + 10,
                _ => 0xff,
            };
            i += 1
        }
        buf
    };
    UNHEX_TABLE[x as usize]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AsciiCase {
    Lower = 0,
    Upper = 1,
}

/// A fixed-length hex string
#[derive(Clone, PartialEq, Eq)]
#[repr(C, align(2))]
pub struct Hex<const N: usize>([u8; N]);

impl<const N: usize> Hex<N> {
    /// Returns [`Hex<N>`](Hex)
    ///
    /// # Safety
    /// This function requires:
    ///
    /// + for all byte in `bytes`, the byte matches `b'0'..=b'9'|b'a'..=b'f'|b'A'..=b'F'`.
    ///
    #[inline]
    pub const unsafe fn new_unchecked(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    #[inline]
    pub const fn into_bytes(self) -> [u8; N] {
        self.0
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }

    #[inline]
    pub const fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }
}

impl<const N: usize> fmt::Debug for Hex<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Debug>::fmt(self.as_str(), f)
    }
}
