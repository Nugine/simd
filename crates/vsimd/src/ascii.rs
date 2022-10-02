use crate::mask::{mask8x32_any, u8x32_highbit_any};
use crate::pod::align;
use crate::tools::{read, slice, slice_parts, unroll};
use crate::vector::V256;
use crate::{Scalable, SIMD256};

use core::ops::Not;

pub mod multiversion {
    use super::*;

    crate::simd_dispatch! (
        name        = is_ascii_ct,
        signature   = fn(data: &[u8]) -> bool,
        fallback    = {is_ascii_ct_fallback},
        simd        = {is_ascii_ct_simd},
        safety      = {},
        visibility  = {pub},
    );

    crate::simd_dispatch!(
        name        = find_non_ascii_whitespace,
        signature   = fn(data: &[u8]) -> usize,
        fallback    = {find_non_ascii_whitespace_fallback},
        simd        = {find_non_ascii_whitespace_simd},
        safety      = {},
        visibility  = {pub},
    );
}

/// An enum type which represents the case of ascii letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiCase {
    /// a-z are lower case letters.
    Lower,
    /// A-Z are upper case letters.
    Upper,
}

#[inline(always)]
#[must_use]
pub fn is_ascii_ct_fallback(data: &[u8]) -> bool {
    let mut ans = 0;
    unroll(data, 8, |&x| ans |= x);
    ans < 0x80
}

#[inline(always)]
pub fn is_ascii_ct_simd<S: SIMD256>(s: S, data: &[u8]) -> bool {
    let (prefix, middle, suffix) = align::<_, V256>(data);

    let mut ans = is_ascii_ct_fallback(prefix);

    let mut mask = s.v256_create_zero();
    unroll(middle, 8, |&chunk| mask = s.v256_or(mask, chunk));
    ans &= u8x32_highbit_any(s, mask).not();

    ans &= is_ascii_ct_fallback(suffix);
    ans
}

#[inline(always)]
#[must_use]
pub fn lookup_ascii_whitespace(c: u8) -> u8 {
    const TABLE: &[u8; 256] = &{
        let mut ans = [0; 256];
        let mut i: u8 = 0;
        loop {
            ans[i as usize] = if i.is_ascii_whitespace() { 0xff } else { 0 };
            if i == 255 {
                break;
            }
            i += 1;
        }
        ans
    };
    unsafe { *TABLE.get_unchecked(c as usize) }
}

#[inline(always)]
fn has_ascii_whitespace_u8x32<S: SIMD256>(s: S, x: V256) -> bool {
    // ASCII whitespaces
    // TAB      0x09    00001001
    // LF       0x0a    00001010
    // FF       0x0c    00001100
    // CR       0x0d    00001101
    // SPACE    0x20    00010000
    //

    const LUT: V256 = V256::double_bytes([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0xff, 0xff, 0x00, 0xff, 0xff, 0x00, 0x00, //
    ]);

    // m1 = {{byte is SPACE}}x32
    let m1 = s.u8x32_eq(x, s.u8x32_splat(0x20));

    // m2 = {{low half is activated}}x32
    let m2 = s.u8x16x2_swizzle(LUT, x);

    // m3 = {{high half is zero}}x32
    let m3 = s.u8x32_eq(s.v256_and(x, s.u8x32_splat(0xf0)), s.v256_create_zero());

    // any(m1 | (m2 & m3))
    mask8x32_any(s, s.v256_or(m1, s.v256_and(m2, m3)))
}

#[inline(always)]
#[must_use]
pub fn find_non_ascii_whitespace_fallback(data: &[u8]) -> usize {
    unsafe {
        let len = data.len();
        let mut src = data.as_ptr();
        let base = src;

        const L: usize = 8;
        let end = src.add(len / L * L);
        while src < end {
            let mut flag = 0;
            let mut i = 0;
            while i < L {
                flag |= lookup_ascii_whitespace(read(src, i));
                i += 1;
            }
            if flag != 0 {
                break;
            }
            src = src.add(L);
        }

        let end = base.add(len);
        while src < end {
            if lookup_ascii_whitespace(src.read()) != 0 {
                break;
            }
            src = src.add(1);
        }

        src.offset_from(base) as usize
    }
}

#[inline(always)]
pub fn find_non_ascii_whitespace_simd<S: SIMD256>(s: S, data: &[u8]) -> usize {
    unsafe {
        let (mut src, len) = slice_parts(data);
        let base = src;

        let end = src.add(len / 32 * 32);
        while src < end {
            let x = s.v256_load_unaligned(src);
            if has_ascii_whitespace_u8x32(s, x) {
                break;
            }
            src = src.add(32);
        }

        let checked_len = src.offset_from(base) as usize;
        let pos = find_non_ascii_whitespace_fallback(slice(src, len - checked_len));
        checked_len + pos
    }
}

#[inline(always)]
fn convert_ascii_case<S: Scalable<V>, V: Copy, const C: u8>(s: S, x: V) -> V {
    assert!(matches!(C, b'A' | b'a'));
    let x1 = s.u8xn_sub(x, s.u8xn_splat(C + 0x80));
    let x2 = s.i8xn_lt(x1, s.i8xn_splat(-0x80 + 26));
    let x3 = s.and(x2, s.u8xn_splat(0x20));
    s.xor(x, x3)
}

#[inline(always)]
pub fn to_ascii_lowercase<S: Scalable<V>, V: Copy>(s: S, x: V) -> V {
    convert_ascii_case::<S, V, b'A'>(s, x)
}

#[inline(always)]
pub fn to_ascii_uppercase<S: Scalable<V>, V: Copy>(s: S, x: V) -> V {
    convert_ascii_case::<S, V, b'a'>(s, x)
}

#[cfg(test)]
mod algorithm {
    use crate::algorithm::*;

    #[ignore]
    #[test]
    fn convert_case() {
        let convert = |c: u8, shift: u8| {
            let x1 = c.wrapping_sub(shift + 0x80);
            let x2 = i8_lt(x1 as i8, -0x80 + 26);
            let x3 = x2 & 0x20;
            c ^ x3
        };
        let to_upper = |c: u8| convert(c, b'a');
        let to_lower = |c: u8| convert(c, b'A');

        print_fn_table(|c| c.is_ascii_lowercase(), to_upper);
        print_fn_table(|c| c.is_ascii_uppercase(), to_lower);

        for c in 0..=255u8 {
            assert_eq!(to_upper(c), c.to_ascii_uppercase());
            assert_eq!(to_lower(c), c.to_ascii_lowercase());
        }
    }
}
