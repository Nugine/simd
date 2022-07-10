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
