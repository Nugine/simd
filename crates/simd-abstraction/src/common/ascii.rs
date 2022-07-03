use crate::tools::{Bytes32, Load};
use crate::traits::SIMD256;

#[inline]
pub fn is_ascii_ct<S: SIMD256>(s: S, data: &[u8]) -> bool {
    let (prefix, chunks, suffix) = unsafe { data.align_to::<Bytes32>() };

    let mut ans;

    {
        ans = check_fallback_ct(prefix);
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
        ans &= check_fallback_ct(suffix);
    }

    ans
}

#[inline(always)]
fn check_fallback_ct(data: &[u8]) -> bool {
    let mut ans = 0;
    for &x in data {
        ans |= x;
    }
    ans < 0x80
}
