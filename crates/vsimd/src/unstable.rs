use core::simd::{Simd, SimdElement};

#[inline(always)]
pub fn splat<T, const N: usize>(x: T) -> Simd<T, N>
where
    T: SimdElement,
{
    Simd::splat(x)
}
