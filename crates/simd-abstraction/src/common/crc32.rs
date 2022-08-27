use crate::isa::InstructionSet;
use crate::scalar::align;
use crate::tools::unroll;

pub unsafe trait CRC32<const P: u32>: InstructionSet {
    fn crc32_u8(self, crc: u32, value: u8) -> u32;
    fn crc32_u16(self, crc: u32, value: u16) -> u32;
    fn crc32_u32(self, crc: u32, value: u32) -> u32;
    fn crc32_u64(self, crc: u32, value: u64) -> u32;
}

pub const POLYNOMIAL_CRC32_IEEE: u32 = 0x04C11DB7;
pub const POLYNOMIAL_CRC32C: u32 = 0x1EDC6F41;

#[inline]
pub fn compute<S, const P: u32>(s: S, init: u32, data: &[u8]) -> u32
where
    S: CRC32<P>,
{
    let mut crc = !init;

    let (prefix, middle, suffix) = align::<u8, u64>(data);

    let fold_u8 = |crc, value| s.crc32_u8(crc, value);
    crc = fold_copied(prefix, crc, fold_u8);

    unroll(middle, 8, |&value| crc = s.crc32_u64(crc, value));

    crc = fold_copied(suffix, crc, fold_u8);

    !crc
}

fn fold_copied<T: Copy, B>(slice: &[T], init: B, f: impl Fn(B, T) -> B) -> B {
    slice.iter().copied().fold(init, f)
}

mod spec {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    mod x86 {
        use super::super::*;

        use crate::arch::x86::SSE42;

        #[cfg(target_arch = "x86")]
        use core::arch::x86::*;

        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::*;

        unsafe impl CRC32<POLYNOMIAL_CRC32C> for SSE42 {
            #[inline(always)]
            fn crc32_u8(self, crc: u32, value: u8) -> u32 {
                unsafe { _mm_crc32_u8(crc, value) }
            }

            #[inline(always)]
            fn crc32_u16(self, crc: u32, value: u16) -> u32 {
                unsafe { _mm_crc32_u16(crc, value) }
            }

            #[inline(always)]
            fn crc32_u32(self, crc: u32, value: u32) -> u32 {
                unsafe { _mm_crc32_u32(crc, value) }
            }

            #[inline(always)]
            fn crc32_u64(self, crc: u32, value: u64) -> u32 {
                unsafe { _mm_crc32_u64(crc as u64, value) as u32 }
            }
        }
    }

    #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
    mod aarch64 {
        use super::super::{CRC32, POLYNOMIAL_CRC32C, POLYNOMIAL_CRC32_IEEE};

        use crate::arch::arm;

        use core::arch::aarch64::*;

        unsafe impl CRC32<POLYNOMIAL_CRC32_IEEE> for arm::CRC32 {
            #[inline(always)]
            fn crc32_u8(self, crc: u32, value: u8) -> u32 {
                unsafe { __crc32b(crc, value) }
            }

            #[inline(always)]
            fn crc32_u16(self, crc: u32, value: u16) -> u32 {
                unsafe { __crc32h(crc, value) }
            }

            #[inline(always)]
            fn crc32_u32(self, crc: u32, value: u32) -> u32 {
                unsafe { __crc32w(crc, value) }
            }

            #[inline(always)]
            fn crc32_u64(self, crc: u32, value: u64) -> u32 {
                unsafe { __crc32d(crc, value) }
            }
        }

        unsafe impl CRC32<POLYNOMIAL_CRC32C> for arm::CRC32 {
            #[inline(always)]
            fn crc32_u8(self, crc: u32, value: u8) -> u32 {
                unsafe { __crc32cb(crc, value) }
            }

            #[inline(always)]
            fn crc32_u16(self, crc: u32, value: u16) -> u32 {
                unsafe { __crc32ch(crc, value) }
            }

            #[inline(always)]
            fn crc32_u32(self, crc: u32, value: u32) -> u32 {
                unsafe { __crc32cw(crc, value) }
            }

            #[inline(always)]
            fn crc32_u64(self, crc: u32, value: u64) -> u32 {
                unsafe { __crc32cd(crc, value) }
            }
        }
    }
}
