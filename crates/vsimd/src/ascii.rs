use crate::mask::mask8x32_any;
use crate::scalar::align32;
use crate::tools::{is_same_type, read, unroll};
use crate::{AVX2, SIMD256, SSE41, V256, WASM128};

#[cfg(feature = "unstable")]
use crate::NEON;

pub mod multiversion {
    use super::*;

    crate::simd_dispatch! (
        name        = is_ascii_ct,
        signature   = fn(data: &[u8]) -> bool,
        fallback    = {is_ascii_ct_fallback},
        simd        = {is_ascii_ct_simd},
        safety      = {},
    );

    crate::simd_dispatch!(
        name        = find_non_ascii_whitespace,
        signature   = fn(data: &[u8]) -> usize,
        fallback    = {find_non_ascii_whitespace_fallback},
        simd        = {find_non_ascii_whitespace_simd},
        safety      = {},
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AsciiCase {
    Lower = 0,
    Upper = 1,
}

#[inline]
#[must_use]
pub fn is_ascii_ct_fallback(data: &[u8]) -> bool {
    let mut ans = 0;
    unroll(data, 8, |&x| ans |= x);
    ans < 0x80
}

#[inline]
pub fn is_ascii_ct_simd<S: SIMD256>(s: S, data: &[u8]) -> bool {
    let (prefix, middle, suffix) = align32(data);

    let mut ans = is_ascii_ct_fallback(prefix);

    let mut mask = s.v256_create_zero();
    unroll(middle, 8, |&chunk| mask = s.v256_or(mask, chunk));
    ans &= is_ascii_u8x32(s, mask);

    ans &= is_ascii_ct_fallback(suffix);
    ans
}

#[inline(always)]
fn is_ascii_u8x32<S: SIMD256>(s: S, x: V256) -> bool {
    if is_same_type::<S, AVX2>() {
        return s.u8x32_bitmask(x) == 0;
    }

    if is_same_type::<S, SSE41>() || is_same_type::<S, WASM128>() {
        let x = x.to_v128x2();
        return s.u8x16_bitmask(s.v128_or(x.0, x.1)) == 0;
    }

    #[cfg(feature = "unstable")]
    if is_same_type::<S, NEON>() {
        let x = x.to_v128x2();
        let x = s.v128_or(x.0, x.1);

        if cfg!(target_arch = "arm") {
            return s.v128_all_zero(s.i32x4_lt(x, s.v128_create_zero()));
        }

        if cfg!(target_arch = "aarch64") {
            return s.u8x16_reduce_max(x) < 0x80;
        }

        unreachable!()
    }

    {
        // generic
        unimplemented!()
    }
}

#[inline(always)]
fn lookup_ascii_whitespace(c: u8) -> u8 {
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

#[inline]
pub unsafe fn remove_ascii_whitespace_fallback(data: *mut u8, len: usize) -> usize {
    let mut src: *const u8 = data;
    let mut dst: *mut u8 = data;
    let end: *const u8 = data.add(len);

    while src < end {
        let byte = src.read();
        if lookup_ascii_whitespace(byte) == 0 {
            dst.write(byte);
            dst = dst.add(1);
        }
        src = src.add(1);
    }

    dst.offset_from(data) as usize
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

#[inline]
#[must_use]
pub fn find_non_ascii_whitespace_fallback(data: &[u8]) -> usize {
    unsafe {
        let len = data.len();
        let mut src = data.as_ptr();

        const UNROLL: usize = 8;
        let unroll_end = src.add(len / UNROLL * UNROLL);
        while src < unroll_end {
            let mut flag = 0;
            for i in 0..UNROLL {
                flag |= lookup_ascii_whitespace(read(src, i));
            }
            if flag != 0 {
                break;
            }
            src = src.add(UNROLL);
        }

        let real_end = data.as_ptr().add(len);
        while src < real_end {
            if lookup_ascii_whitespace(src.read()) != 0 {
                break;
            }
            src = src.add(1);
        }

        src.offset_from(data.as_ptr()) as usize
    }
}

#[inline]
pub fn find_non_ascii_whitespace_simd<S: SIMD256>(s: S, data: &[u8]) -> usize {
    let (prefix, middle, suffix) = align32(data);

    let mut pos: usize = 0;

    {
        let offset = find_non_ascii_whitespace_fallback(prefix);
        pos = pos.wrapping_add(offset);
        if offset != prefix.len() {
            return pos;
        }
    }

    for chunk in middle {
        if has_ascii_whitespace_u8x32(s, *chunk) {
            let offset = find_non_ascii_whitespace_fallback(chunk.as_bytes());
            pos += offset;
            return pos;
        }
        pos += 32;
    }

    {
        let offset = find_non_ascii_whitespace_fallback(suffix);
        pos = pos.wrapping_add(offset);
    }

    pos
}
