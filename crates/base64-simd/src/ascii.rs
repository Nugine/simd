use vsimd::mask::mask8x32_any;
use vsimd::tools::{read, slice_parts};
use vsimd::vector::V256;
use vsimd::SIMD256;

#[inline(always)]
#[must_use]
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
pub unsafe fn find_non_ascii_whitespace_fallback(mut src: *const u8, len: usize) -> usize {
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

#[inline(always)]
pub unsafe fn find_non_ascii_whitespace_simd<S: SIMD256>(s: S, mut src: *const u8, len: usize) -> usize {
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
    let pos = find_non_ascii_whitespace_fallback(src, len - checked_len);
    checked_len + pos
}

#[inline(always)]
#[must_use]
pub fn find_non_ascii_whitespace(data: &[u8]) -> usize {
    let (src, len) = slice_parts(data);
    unsafe { crate::multiversion::find_non_ascii_whitespace::auto(src, len) }
}

#[inline(always)]
#[must_use]
pub unsafe fn remove_ascii_whitespace_fallback(mut src: *const u8, len: usize, mut dst: *mut u8) -> usize {
    let dst_base = dst;

    let end = src.add(len);
    while src < end {
        let x = src.read();
        if lookup_ascii_whitespace(x) == 0 {
            dst.write(x);
            dst = dst.add(1);
        }
        src = src.add(1);
    }

    dst.offset_from(dst_base) as usize
}

#[inline(always)]
#[must_use]
pub fn remove_ascii_whitespace_inplace(data: &mut [u8]) -> &mut [u8] {
    let pos = find_non_ascii_whitespace(data);
    debug_assert!(pos <= data.len());

    if pos == data.len() {
        return data;
    }

    unsafe {
        let len = data.len() - pos;
        let dst = data.as_mut_ptr().add(pos);
        let src = dst;

        let rem = remove_ascii_whitespace_fallback(src, len, dst);
        debug_assert!(rem <= len);

        data.get_unchecked_mut(..(pos + rem))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_ascii_whitespace() {
        let cases = [
            "abcd",
            "ab\tcd",
            "ab\ncd",
            "ab\x0Ccd",
            "ab\rcd",
            "ab cd",
            "ab\t\n\x0C\r cd",
            "ab\t\n\x0C\r =\t\n\x0C\r =\t\n\x0C\r ",
        ];
        for case in cases {
            let mut buf = case.to_owned().into_bytes();
            let expected = {
                let mut v = buf.clone();
                v.retain(|c| !c.is_ascii_whitespace());
                v
            };
            let ans = remove_ascii_whitespace_inplace(&mut buf);
            assert_eq!(ans, &*expected, "case = {case:?}");
        }
    }
}
