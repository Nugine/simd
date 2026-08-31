use vsimd::isa::{AVX2, SSSE3, WASM128};
use vsimd::tools::slice_parts;
use vsimd::vector::V128;
use vsimd::{matches_isa, Scalable, POD, SIMD256};

use core::ops::Not;

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
fn ascii_whitespace_mask<S: Scalable<V>, V: POD>(s: S, x: V) -> V {
    // ASCII whitespaces
    // TAB      0x09    00001001
    // LF       0x0a    00001010
    // FF       0x0c    00001100
    // CR       0x0d    00001101
    // SPACE    0x20    00010000
    //

    // m1 = {{byte in 0x09..=0x0d}}x32
    let m1 = s.i8xn_lt(s.u8xn_sub(x, s.u8xn_splat(0x89)), s.i8xn_splat(-128 + 5));

    // m2 = {{byte == 0x0b}}
    let m2 = s.u8xn_eq(x, s.u8xn_splat(0x0b));

    // m3 = {{byte is SPACE}}
    let m3 = s.u8xn_eq(x, s.u8xn_splat(0x20));

    // (m1 & !m2) | m3
    s.or(s.andnot(m1, m2), m3)
}

#[inline(always)]
fn has_ascii_whitespace<S: Scalable<V>, V: POD>(s: S, x: V) -> bool {
    s.mask8xn_any(ascii_whitespace_mask(s, x))
}

#[inline(always)]
unsafe fn find_non_ascii_whitespace_short(mut src: *const u8, len: usize) -> usize {
    let base = src;
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
pub unsafe fn find_non_ascii_whitespace_fallback(src: *const u8, len: usize) -> usize {
    find_non_ascii_whitespace_short(src, len)
}

#[inline(always)]
pub unsafe fn find_non_ascii_whitespace_simd<S: SIMD256>(s: S, mut src: *const u8, len: usize) -> usize {
    let base = src;

    if matches_isa!(S, AVX2) {
        let end = src.add(len / 32 * 32);
        while src < end {
            let x = s.v256_load_unaligned(src);
            if has_ascii_whitespace(s, x) {
                break;
            }
            src = src.add(32);
        }
        if (len % 32) >= 16 {
            let x = s.v128_load_unaligned(src);
            if has_ascii_whitespace(s, x).not() {
                src = src.add(16);
            }
        }
    } else {
        let end = src.add(len / 16 * 16);
        while src < end {
            let x = s.v128_load_unaligned(src);
            if has_ascii_whitespace(s, x) {
                break;
            }
            src = src.add(16);
        }
    }

    let checked_len = src.offset_from(base) as usize;
    let pos = find_non_ascii_whitespace_short(src, len - checked_len);
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

/// For every 8-bit removal mask, the byte indices that survive, in order, packed little-endian.
/// Unused lanes are zero; they are overwritten by the following block's bytes.
const COMPRESS_INDEX: &[u64; 256] = &{
    let mut table = [0u64; 256];
    let mut m = 0;
    while m < 256 {
        let mut indices = [0u8; 8];
        let mut kept = 0;
        let mut i = 0;
        while i < 8 {
            if (m >> i) & 1 == 0 {
                indices[kept] = i as u8;
                kept += 1;
            }
            i += 1;
        }
        table[m] = u64::from_le_bytes(indices);
        m += 1;
    }
    table
};

/// Compacts a 16-byte block with `vpshufb`-style byte shuffles: each 8-byte half is gathered by
/// one table-driven swizzle, then written back with two overlapping 8-byte stores.
///
/// Both stores stay inside `dst[..16]`, which the caller has already proven writable.
#[inline(always)]
unsafe fn compress_block16<S: SIMD256>(s: S, x: V128, mask: u16, dst: *mut u8) -> usize {
    let lo_mask = (mask & 0xff) as usize;
    let hi_mask = (mask >> 8) as usize;

    // The high half indexes bytes 8..16, so every index is shifted by 8; no lane can carry.
    let lo_index = *COMPRESS_INDEX.get_unchecked(lo_mask);
    let hi_index = COMPRESS_INDEX
        .get_unchecked(hi_mask)
        .wrapping_add(0x0808_0808_0808_0808);

    let mut index = [0u8; 16];
    index[..8].copy_from_slice(&lo_index.to_le_bytes());
    index[8..].copy_from_slice(&hi_index.to_le_bytes());

    let (lo, hi) = s.u8x16_swizzle(x, V128::from_bytes(index)).to_v64x2();

    let lo_kept = 8 - lo_mask.count_ones() as usize;
    let hi_kept = 8 - hi_mask.count_ones() as usize;

    dst.cast::<u64>().write_unaligned(lo.to_u64());
    dst.add(lo_kept).cast::<u64>().write_unaligned(hi.to_u64());

    lo_kept + hi_kept
}

/// Compacts one 16-byte block, returning the number of retained bytes written to `dst`.
///
/// `dst` may trail `src` within the same allocation. The block is loaded before anything is
/// stored, so a clean block is copied with a single vector store even when the ranges overlap.
#[inline(always)]
unsafe fn remove_ascii_whitespace_block16<S: SIMD256>(s: S, src: *const u8, dst: *mut u8) -> usize {
    let x = s.v128_load_unaligned(src);
    let m = ascii_whitespace_mask(s, x);

    if s.mask8xn_any(m).not() {
        s.v128_store_unaligned(dst, x);
        return 16;
    }

    // `u8x16_bitmask` is unimplemented for NEON and `u8x16_swizzle` needs SSSE3, so other
    // targets keep the scalar compaction tail.
    if matches_isa!(S, SSSE3 | WASM128) {
        return compress_block16(s, x, s.u8x16_bitmask(m), dst);
    }

    remove_ascii_whitespace_fallback(src, 16, dst)
}

#[inline(always)]
#[must_use]
pub(crate) unsafe fn remove_ascii_whitespace_simd<S: SIMD256>(
    s: S,
    mut src: *const u8,
    len: usize,
    mut dst: *mut u8,
) -> usize {
    let dst_base = dst;
    let end = src.add(len);

    if matches_isa!(S, AVX2) {
        let block_end = src.add(len / 32 * 32);
        while src < block_end {
            let x = s.v256_load_unaligned(src);
            if has_ascii_whitespace(s, x) {
                dst = dst.add(remove_ascii_whitespace_block16(s, src, dst));
                dst = dst.add(remove_ascii_whitespace_block16(s, src.add(16), dst));
            } else {
                s.v256_store_unaligned(dst, x);
                dst = dst.add(32);
            }
            src = src.add(32);
        }
    }

    {
        let rem = end.offset_from(src) as usize;
        let block_end = src.add(rem / 16 * 16);
        while src < block_end {
            dst = dst.add(remove_ascii_whitespace_block16(s, src, dst));
            src = src.add(16);
        }
    }

    let rem = end.offset_from(src) as usize;
    dst = dst.add(remove_ascii_whitespace_fallback(src, rem, dst));

    dst.offset_from(dst_base) as usize
}

/// Removes ASCII whitespace from `src[..len]`, writing the retained bytes to `dst`.
///
/// # Safety
/// `src[..len]` must be readable and `dst[..len]` writable. `dst` may equal or trail `src` in
/// the same allocation, but must not lead it.
#[inline(always)]
#[must_use]
pub(crate) unsafe fn remove_ascii_whitespace(src: *const u8, len: usize, dst: *mut u8) -> usize {
    crate::multiversion::remove_ascii_whitespace::auto(src, len, dst)
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

        let rem = remove_ascii_whitespace(src, len, dst);
        debug_assert!(rem <= len);

        data.get_unchecked_mut(..(pos + rem))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_remove_ascii_whitespace() {
        let cases = [
            "\0\0\0\0",
            "abcd",
            "ab\tcd",
            "ab\ncd",
            "ab\x0Ccd",
            "ab\rcd",
            "ab cd",
            "ab\t\n\x0C\r cd",
            "ab\t\n\x0C\r =\t\n\x0C\r =\t\n\x0C\r ",
        ];

        let check = |case: &str, repeat: usize| {
            let mut buf = case.repeat(repeat).into_bytes();
            let expected = {
                let mut v = buf.clone();
                v.retain(|c| !c.is_ascii_whitespace());
                v
            };
            let ans = remove_ascii_whitespace_inplace(&mut buf);
            assert_eq!(ans, &*expected, "case = {case:?}");
        };

        for case in cases {
            check(case, 1);

            if cfg!(not(miri)) {
                check(case, 10);
            }
        }
    }
}

#[cfg(test)]
mod algorithm {
    #[cfg_attr(
        any(miri, not(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))),
        ignore
    )]
    #[test]
    fn is_ascii_whitespace() {
        for x in 0..=255u8 {
            let m1 = (x.wrapping_sub(0x89) as i8) < (-128 + 5);
            let m2 = x == 0x0b;
            let m3 = x == 0x20;
            let ans = (m1 && !m2) || m3;
            assert_eq!(ans, x.is_ascii_whitespace());
        }
    }
}
