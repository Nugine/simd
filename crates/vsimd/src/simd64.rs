use crate::isa::InstructionSet;
use crate::vector::V64;

#[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
use crate::isa::NEON;

#[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
use core::mem::transmute as t;

#[cfg(all(feature = "unstable", target_arch = "arm"))]
use core::arch::arm::*;

#[cfg(all(feature = "unstable", target_arch = "aarch64"))]
use core::arch::aarch64::*;

pub unsafe trait SIMD64: InstructionSet {
    #[inline(always)]
    #[must_use]
    fn u8x8_unzip_even(self, a: V64, b: V64) -> V64 {
        #[cfg(all(feature = "unstable", target_arch = "arm"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vuzp_u8(t(a), t(b)).0) };
        }
        #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
        if is_subtype!(Self, NEON) {
            return unsafe { t(vuzp1_u8(t(a), t(b))) };
        }
        {
            let _ = (a, b);
            unreachable!()
        }
    }
}
