use vsimd::tools::slice_parts;

#[inline]
pub unsafe fn swap_endianness(src: *const u32, len: usize, dst: *mut u32) {
    vsimd::bswap::bswap_fallback(src, len, dst);
}

#[inline]
pub fn is_utf32le_ct(data: &[u32]) -> bool {
    unsafe {
        let mut flag = true;

        let (mut src, len) = slice_parts(data);
        let end = src.add(len);
        while src < end {
            let x = src.read();
            flag &= is_unicode_scalar_value(x);
            src = src.add(1);
        }

        flag
    }
}

/// See [`char::from_u32`](core::char::from_u32)
#[inline(always)]
fn is_unicode_scalar_value(x: u32) -> bool {
    (x ^ 0xD800).wrapping_sub(0x800) < (0x11_0000 - 0x800)
}
