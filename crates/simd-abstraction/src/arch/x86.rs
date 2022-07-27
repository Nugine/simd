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
mod sse42;
