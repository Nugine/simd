use crate::tools::{Bytes32, Load};
use crate::traits::SIMD256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AsciiCase {
    Lower = 0,
    Upper = 1,
}

#[inline]
pub fn is_ascii_ct_simd<S: SIMD256>(s: S, data: &[u8]) -> bool {
    let (prefix, chunks, suffix) = unsafe { data.align_to::<Bytes32>() };

    let mut ans;

    {
        ans = is_ascii_ct_fallback(prefix);
    }

    {
        let mut mask = s.v256_create_zero();
        for chunk in chunks {
            let a = s.load(chunk);
            mask = s.v256_or(mask, a);
        }
        ans &= s.v256_all_zero(s.i8x32_cmp_lt(mask, s.v256_create_zero()));
    }

    {
        ans &= is_ascii_ct_fallback(suffix);
    }

    ans
}

#[inline]
pub fn is_ascii_ct_fallback(data: &[u8]) -> bool {
    let mut ans = 0;
    for &x in data {
        ans |= x;
    }
    ans < 0x80
}

#[inline(always)]
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

#[inline]
pub fn find_non_ascii_whitespace_fallback(data: &[u8]) -> usize {
    unsafe {
        let n = data.len();
        let mut src = data.as_ptr();

        const UNROLL: usize = 8;
        let end = src.add(n / UNROLL * UNROLL);
        while src < end {
            let mut flag = 0;
            for _ in 0..UNROLL {
                flag |= lookup_ascii_whitespace(src.read());
                src = src.add(1)
            }
            if flag != 0 {
                src = src.sub(UNROLL);
                break;
            }
        }

        let end = data.as_ptr().add(n);
        while src < end {
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
    let (prefix, chunks, suffix) = unsafe { data.align_to::<Bytes32>() };

    let mut pos: usize = 0;

    {
        let offset = find_non_ascii_whitespace_fallback(prefix);
        pos = pos.wrapping_add(offset);
        if offset != prefix.len() {
            return pos;
        }
    }

    for chunk in chunks {
        if check_non_ascii_whitespace_u8x32(s, s.load(chunk)) {
            let offset = find_non_ascii_whitespace_fallback(&chunk.0);
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

#[inline(always)]
fn check_non_ascii_whitespace_u8x32<S: SIMD256>(s: S, a: S::V256) -> bool {
    // ASCII whitespaces
    // TAB      0x09    00001001
    // LF       0x0a    00001010
    // FF       0x0c    00001100
    // CR       0x0d    00001101
    // SPACE    0x20    00010000
    //

    const LUT: &Bytes32 = &Bytes32([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
        0xff, 0x00, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, //
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
        0xff, 0x00, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, //
    ]);

    let lut: _ = s.load(LUT);

    let m1: _ = s.u8x16x2_swizzle(lut, a);
    let m2: _ = s.v256_and(a, s.u8x32_splat(0xf0));
    let m3: _ = s.i8x32_cmp_eq(s.v256_or(m1, m2), s.v256_create_zero());
    let m4: _ = s.i8x32_cmp_eq(a, s.i8x32_splat(0x20));
    let m5: _ = s.v256_or(m3, m4);

    !s.v256_all_zero(m5)
}
