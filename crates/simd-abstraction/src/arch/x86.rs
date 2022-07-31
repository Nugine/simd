use crate::isa::{InstructionSet, SIMD128, SIMD256};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

define_isa!(SSE41, "sse4.1", is_x86_feature_detected);
define_isa!(SSE42, "sse4.2", is_x86_feature_detected);
define_isa!(AVX2, "avx2", is_x86_feature_detected);

mod avx2;
mod sse41;

impl SSE42 {
    #[inline(always)]
    fn sse41(self) -> SSE41 {
        unsafe { SSE41::new() }
    }
}

impl AVX2 {
    #[inline(always)]
    fn sse41(self) -> SSE41 {
        unsafe { SSE41::new() }
    }
}

inherit_simd128!(SSE42, SSE41, sse41);
inherit_simd256!(SSE42, SSE41, sse41);

inherit_simd128!(AVX2, SSE41, sse41);
