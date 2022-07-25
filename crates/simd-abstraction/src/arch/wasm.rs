use crate::traits::{self, InstructionSet};

#[cfg(target_arch = "wasm32")]
use core::arch::wasm32::*;

#[derive(Debug, Clone, Copy)]
pub struct SIMD128(());

unsafe impl InstructionSet for SIMD128 {
    #[inline(always)]
    fn is_enabled() -> bool {
        cfg!(target_feature = "simd128")
    }

    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }
}

unsafe impl traits::SIMD128 for SIMD128 {
    type V128 = v128;

    #[inline(always)]
    unsafe fn v128_load(self, addr: *const u8) -> Self::V128 {
        debug_assert_ptr_align!(addr, 16);
        v128_load(addr.cast())
    }

    #[inline(always)]
    unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128 {
        v128_load(addr.cast())
    }

    #[inline(always)]
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
        v128_store(addr.cast(), a)
    }

    #[inline(always)]
    fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        v128_or(a, b)
    }

    #[inline(always)]
    fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        v128_and(a, b)
    }

    #[inline(always)]
    fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
        unsafe { core::mem::transmute(a) }
    }

    #[inline(always)]
    fn v128_create_zero(self) -> Self::V128 {
        u8x16_splat(0)
    }

    #[inline(always)]
    fn v128_all_zero(self, a: Self::V128) -> bool {
        !v128_any_true(a)
    }

    #[inline(always)]
    fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        v128_andnot(a, b)
    }

    #[inline(always)]
    fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        v128_xor(a, b)
    }

    #[inline(always)]
    fn u8x16_splat(self, x: u8) -> Self::V128 {
        u8x16_splat(x)
    }

    #[inline(always)]
    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_swizzle(a, b)
    }

    #[inline(always)]
    fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_add(a, b)
    }

    #[inline(always)]
    fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_sub(a, b)
    }

    #[inline(always)]
    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_sub_sat(a, b)
    }

    #[inline(always)]
    fn u8x16_any_zero(self, a: Self::V128) -> bool {
        !u8x16_all_true(a)
    }

    #[inline(always)]
    fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_min(a, b)
    }

    #[inline(always)]
    fn i8x16_splat(self, x: i8) -> Self::V128 {
        i8x16_splat(x)
    }

    #[inline(always)]
    fn i8x16_cmp_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i8x16_lt(a, b)
    }

    #[inline(always)]
    fn i8x16_cmp_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i8x16_eq(a, b)
    }

    #[inline(always)]
    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        u16x8_shl(a, IMM8 as u32)
    }

    #[inline(always)]
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        u16x8_shr(a, IMM8 as u32)
    }

    #[inline(always)]
    fn u16x8_splat(self, x: u16) -> Self::V128 {
        u16x8_splat(x)
    }

    #[inline(always)]
    fn u32x4_splat(self, x: u32) -> Self::V128 {
        u32x4_splat(x)
    }

    #[inline(always)]
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        u32x4_shl(a, IMM8 as u32)
    }

    #[inline(always)]
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        u32x4_shr(a, IMM8 as u32)
    }

    #[inline(always)]
    fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u32x4_add(a, b)
    }

    #[inline(always)]
    fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u32x4_sub(a, b)
    }
}

unsafe impl traits::SIMD256 for SIMD128 {
    type V256 = (v128, v128);

    #[inline(always)]
    fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256 {
        (a, b)
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
        let a0 = u16x8_extend_low_u8x16(a);
        let a1 = u16x8_extend_high_u8x16(a);
        self.v256_from_v128x2(a0, a1)
    }

    #[inline(always)]
    fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
        let a = self.v256_to_v128x2(a);
        u64x2_shuffle::<0, 2>(a.0, a.1)
    }
}
