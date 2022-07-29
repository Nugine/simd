use crate::isa::{InstructionSet, SIMD128, SIMD256};

#[cfg(target_arch = "arm")]
use core::arch::arm::*;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[cfg(target_arch = "arm")]
define_isa!(NEON, "neon", is_arm_feature_detected);

#[cfg(target_arch = "aarch64")]
define_isa!(NEON, "neon", is_aarch64_feature_detected);

#[cfg(target_arch = "aarch64")]
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
        #[cfg(target_arch = "arm")]
        unsafe {
            let a0 = vreinterpretq_u64_u8(a);
            let a1 = vorr_u64(vget_low_u64(a0), vget_high_u64(a0));
            vget_lane_u64::<0>(a1) == 0
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            vmaxvq_u8(a) == 0
        }
    }

    #[inline(always)]
    fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { vandq_u8(a, vmvnq_u8(b)) }
    }

    #[inline(always)]
    fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { veorq_u8(a, b) }
    }

    #[inline(always)]
    fn u8x16_splat(self, x: u8) -> Self::V128 {
        unsafe { vdupq_n_u8(x) }
    }

    #[inline(always)]
    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        #[cfg(target_arch = "arm")]
        unsafe {
            let a = uint8x8x2_t(vget_low_u8(a), vget_high_u8(a));
            let (b0, b1) = (vget_low_u8(b), vget_high_u8(b));
            let c0 = vget_lane_u64::<0>(vreinterpret_u64_u8(vtbl2_u8(a, b0)));
            let c1 = vget_lane_u64::<0>(vreinterpret_u64_u8(vtbl2_u8(a, b1)));
            vreinterpretq_u8_u64(vsetq_lane_u64::<1>(c1, vdupq_n_u64(c0)))
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            vqtbl1q_u8(a, b)
        }
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
    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { vqsubq_u8(a, b) }
    }

    #[inline(always)]
    fn u8x16_any_zero(self, a: Self::V128) -> bool {
        #[cfg(target_arch = "arm")]
        unsafe {
            !self.v128_all_zero(vceqq_u8(a, vdupq_n_u8(0)))
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            vminvq_u8(a) == 0
        }
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
    fn i8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe {
            let f = vreinterpretq_s8_u8;
            vcltq_s8(f(a), f(b))
        }
    }

    #[inline(always)]
    fn i8x16_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe { vceqq_u8(a, b) }
    }

    #[inline(always)]
    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { vreinterpretq_u8_u16(vshlq_n_u16::<IMM8>(vreinterpretq_u16_u8(a))) }
    }

    #[inline(always)]
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { vreinterpretq_u8_u16(vshrq_n_u16::<IMM8>(vreinterpretq_u16_u8(a))) }
    }

    #[inline(always)]
    fn u16x8_splat(self, x: u16) -> Self::V128 {
        unsafe { vreinterpretq_u8_u16(vdupq_n_u16(x)) }
    }

    #[inline(always)]
    fn u32x4_splat(self, x: u32) -> Self::V128 {
        unsafe { vreinterpretq_u8_u32(vdupq_n_u32(x)) }
    }

    #[inline(always)]
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { vreinterpretq_u8_u32(vshlq_n_u32::<IMM8>(vreinterpretq_u32_u8(a))) }
    }

    #[inline(always)]
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        unsafe { vreinterpretq_u8_u32(vshrq_n_u32::<IMM8>(vreinterpretq_u32_u8(a))) }
    }

    #[inline(always)]
    fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe {
            let f = vreinterpretq_u32_u8;
            let g = vreinterpretq_u8_u32;
            g(vaddq_u32(f(a), f(b)))
        }
    }

    #[inline(always)]
    fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe {
            let f = vreinterpretq_u32_u8;
            let g = vreinterpretq_u8_u32;
            g(vsubq_u32(f(a), f(b)))
        }
    }

    #[inline(always)]
    fn u32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe {
            let f = vreinterpretq_u32_u8;
            let g = vreinterpretq_u8_u32;
            g(vmaxq_u32(f(a), f(b)))
        }
    }

    #[inline(always)]
    fn u32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe {
            let f = vreinterpretq_u32_u8;
            let g = vreinterpretq_u8_u32;
            g(vcltq_u32(f(a), f(b)))
        }
    }

    #[inline(always)]
    fn i32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        unsafe {
            let f = vreinterpretq_s32_u8;
            let g = vreinterpretq_u8_u32;
            g(vcltq_s32(f(a), f(b)))
        }
    }

    #[inline(always)]
    fn u32x4_bswap(self, a: Self::V128) -> Self::V128 {
        unsafe {
            let f = vreinterpretq_u32_u8;
            let g = vreinterpretq_u8_u32;
            g(vrev64q_u32(f(a)))
        }
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
        #[cfg(target_arch = "arm")]
        unsafe {
            let a0 = vreinterpretq_u8_u16(vmovl_u8(vget_low_u8(a)));
            let a1 = vreinterpretq_u8_u16(vmovl_u8(vget_high_u8(a)));
            uint8x16x2_t(a0, a1)
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let f = vreinterpretq_u8_u16;
            uint8x16x2_t(f(vmovl_u8(vget_low_u8(a))), f(vmovl_high_u8(a)))
        }
    }

    #[inline(always)]
    fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
        #[cfg(target_arch = "arm")]
        unsafe {
            let a0 = vgetq_lane_u64::<0>(vreinterpretq_u64_u8(a.0));
            let a1 = vgetq_lane_u64::<0>(vreinterpretq_u64_u8(a.1));
            vreinterpretq_u8_u64(vsetq_lane_u64::<1>(a1, vdupq_n_u64(a0)))
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let f = vreinterpretq_u64_u8;
            let g = vreinterpretq_u8_u64;
            g(vuzp1q_u64(f(a.0), f(a.1)))
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
