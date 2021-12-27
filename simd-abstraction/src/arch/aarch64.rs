use crate::traits::{self,InstructionSet, SIMD128, SIMD256};

use core::arch::aarch64::*;

define_isa!(NEON, "neon", is_aarch64_feature_detected);
define_isa!(CRC32, "crc", is_aarch64_feature_detected);

unsafe impl SIMD128 for NEON {
    type V128 = uint8x16_t;

    #[inline(always)]
    unsafe fn v128_load(self, addr: *const u8) -> Self::V128 {
        debug_assert_ptr_align!(addr, 16);
        vld1q_u8(addr)
    }

    #[inline(always)]
    unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128 {
        vld1q_u8(addr)
    }

    #[inline(always)]
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
        vst1q_u8(addr, a)
    }

    #[inline(always)]
    fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { vorrq_u8(a, b) }
    }

    #[inline(always)]
    fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { vandq_u8(a, b) }
    }

    #[inline(always)]
    fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
        unsafe { core::mem::transmute(a) }
    }

    #[inline(always)]
    fn v128_create_zero(self) -> Self::V128 {
        unsafe { vdupq_n_u8(0) }
    }

    #[inline(always)]
    fn v128_all_zero(self, a: Self::V128) -> bool {
        unsafe { vmaxvq_u8(a) == 0 }
    }

    #[inline(always)]
    fn u8x16_splat(self, x: u8) -> Self::V128 {
        unsafe { vdupq_n_u8(x) }
    }

    #[inline(always)]
    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { vqtbl1q_u8(a, b) }
    }

    #[inline(always)]
    fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { vaddq_u8(a, b) }
    }

    #[inline(always)]
    fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { vsubq_u8(a, b) }
    }

    #[inline(always)]
    fn u8x16_any_zero(self, a: Self::V128) -> bool {
        unsafe { vminvq_u8(a) == 0 }
    }

    #[inline(always)]
    fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { vminq_u8(a, b) }
    }

    #[inline(always)]
    fn i8x16_splat(self, x: i8) -> Self::V128 {
        unsafe { vreinterpretq_u8_s8(vdupq_n_s8(x)) }
    }

    #[inline(always)]
    fn i8x16_cmp_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe {
            let f = vreinterpretq_s8_u8;
            vcltq_s8(f(a), f(b))
        }
    }

    #[inline(always)]
    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { vreinterpretq_u8_u16(vshlq_n_u16::<IMM8>(vreinterpretq_u16_u8(a))) }
    }

    #[inline(always)]
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { vreinterpretq_u8_u16(vshrq_n_u16::<IMM8>(vreinterpretq_u16_u8(a))) }
    }
}

unsafe impl SIMD256 for NEON {
    type V256 = uint8x16x2_t;

    #[inline(always)]
    fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256 {
        uint8x16x2_t(a, b)
    }

    #[inline(always)]
    fn v256_to_v128x2(self, a: Self::V256) -> (Self::V128, Self::V128) {
        (a.0, a.1)
    }

    #[inline(always)]
    fn v256_to_bytes(self, a: Self::V256) -> [u8; 32] {
        unsafe { core::mem::transmute([a.0, a.1]) }
    }

    #[inline(always)]
    fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256 {
        unsafe {
            let f = vreinterpretq_u8_u16;
            uint8x16x2_t(f(vmovl_u8(vget_low_u8(a))), f(vmovl_high_u8(a)))
        }
    }

    #[inline(always)]
    fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
        unsafe {
            let f = vreinterpretq_u64_u8;
            vreinterpretq_u8_u64(vuzp1q_u64(f(a.0), f(a.1)))
        }
    }

    #[inline(always)]
    unsafe fn v256_load(self, addr: *const u8) -> Self::V256 {
        debug_assert_ptr_align!(addr, 32);
        vld1q_u8_x2(addr)
    }

    #[inline(always)]
    unsafe fn v256_load_unaligned(self, addr: *const u8) -> Self::V256 {
        vld1q_u8_x2(addr)
    }

    #[inline(always)]
    unsafe fn v256_store_unaligned(self, addr: *mut u8, a: Self::V256) {
        vst1q_u8_x2(addr, a)
    }
}

unsafe impl traits::CRC32<{ traits::CRC32IEEE }> for CRC32 {
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

unsafe impl traits::CRC32<{ traits::CRC32C }> for CRC32 {
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
