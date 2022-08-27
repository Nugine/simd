use crate::isa::SIMD256;

/// x: `{{0xff or 0x00}}x32`
#[inline(always)]
pub fn mask8x32_all<S: SIMD256>(s: S, x: S::V256) -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        // x86
        use crate::arch::x86::*;
        use crate::isa::SIMD128;
        use core::mem::transmute_copy;

        if let Some(s) = s.concrete_type::<AVX2>() {
            let x: <AVX2 as SIMD256>::V256 = unsafe { transmute_copy(&x) };
            return s.u8x32_bitmask(x) == u32::MAX;
        }

        if let Some(s) = s.concrete_type::<SSE41>() {
            let x: <SSE41 as SIMD256>::V256 = unsafe { transmute_copy(&x) };
            return s.u8x16_bitmask(s.v128_and(x.0, x.1)) == u16::MAX;
        }
    }
    {
        // generic
        !s.u8x32_any_zero(x)
    }
}
