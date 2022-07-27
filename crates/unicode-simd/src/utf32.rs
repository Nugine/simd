use simd_abstraction::scalar::Bytes32;
use simd_abstraction::tools::unroll;
use simd_abstraction::traits::{SimdLoad, SIMD256};

/// See [`char::from_u32`]
#[inline]
pub fn is_utf32le_ct_fallback(data: &[u32]) -> bool {
    let mut flag = false;
    unroll(data, 8, |&x| {
        flag |= (x ^ 0xD800).wrapping_sub(0x800) >= (0x110000 - 0x800);
    });
    !flag
}

#[inline]
pub fn is_utf32le_ct_simd<S: SIMD256>(s: S, data: &[u32]) -> bool {
    let (prefix, middle, suffix) = unsafe { data.align_to::<Bytes32>() };

    let mut ans = is_utf32le_ct_fallback(prefix);

    {
        let m1 = s.u32x8_splat(0xD800);
        let m2 = s.u32x8_splat(0x800);
        let mut y = s.u32x8_splat(0);

        unroll(middle, 8, |chunk| {
            let x = s.load(chunk);
            let a1 = s.v256_xor(x, m1);
            let a2 = s.u32x8_sub(a1, m2);
            y = s.u32x8_max(y, a2);
        });

        let m3 = s.u32x8_splat(0x110000 - 0x800 - 1);
        ans &= s.v256_all_zero(s.i8x32_cmp_lt(m3, y));
    }

    ans &= is_utf32le_ct_fallback(suffix);

    ans
}
