use crate::isa::SimdLoad;
use crate::isa::{self, InstructionSet};

#[cfg(target_arch = "wasm32")]
use core::arch::wasm32::*;

define_isa!(SIMD128, "simd128");

unsafe impl isa::SIMD128 for SIMD128 {
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
    fn u8x16_any_zero(self, a: Self::V128) -> bool {
        !u8x16_all_true(a)
    }

    #[inline(always)]
    fn u8x16_splat(self, x: u8) -> Self::V128 {
        u8x16_splat(x)
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
    fn u64x2_splat(self, x: u64) -> Self::V128 {
        u64x2_splat(x)
    }

    #[inline(always)]
    fn i8x16_splat(self, x: i8) -> Self::V128 {
        i8x16_splat(x)
    }

    #[inline(always)]
    fn i16x8_splat(self, x: i16) -> Self::V128 {
        i16x8_splat(x)
    }

    #[inline(always)]
    fn i32x4_splat(self, x: i32) -> Self::V128 {
        i32x4_splat(x)
    }

    #[inline(always)]
    fn i64x2_splat(self, x: i64) -> Self::V128 {
        i64x2_splat(x)
    }

    #[inline(always)]
    fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_add(a, b)
    }

    #[inline(always)]
    fn u16x8_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u16x8_add(a, b)
    }

    #[inline(always)]
    fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u32x4_add(a, b)
    }

    #[inline(always)]
    fn u64x2_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u64x2_add(a, b)
    }

    #[inline(always)]
    fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_sub(a, b)
    }

    #[inline(always)]
    fn u16x8_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u16x8_sub(a, b)
    }

    #[inline(always)]
    fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u32x4_sub(a, b)
    }

    #[inline(always)]
    fn u64x2_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u64x2_sub(a, b)
    }

    #[inline(always)]
    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_sub_sat(a, b)
    }

    #[inline(always)]
    fn u16x8_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u16x8_sub_sat(a, b)
    }

    #[inline(always)]
    fn i8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i8x16_sub_sat(a, b)
    }

    #[inline(always)]
    fn i16x8_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i16x8_sub_sat(a, b)
    }

    #[inline(always)]
    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        u16x8_shl(a, IMM8 as u32)
    }

    #[inline(always)]
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        u32x4_shl(a, IMM8 as u32)
    }

    #[inline(always)]
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        u16x8_shr(a, IMM8 as u32)
    }

    #[inline(always)]
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        u32x4_shr(a, IMM8 as u32)
    }

    #[inline(always)]
    fn u8x16_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_eq(a, b)
    }

    #[inline(always)]
    fn u16x8_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u16x8_eq(a, b)
    }

    #[inline(always)]
    fn u32x4_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u32x4_eq(a, b)
    }

    #[inline(always)]
    fn u8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_lt(a, b)
    }

    #[inline(always)]
    fn u16x8_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u16x8_lt(a, b)
    }

    #[inline(always)]
    fn u32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u32x4_lt(a, b)
    }

    #[inline(always)]
    fn i8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i8x16_lt(a, b)
    }

    #[inline(always)]
    fn i16x8_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i16x8_lt(a, b)
    }

    #[inline(always)]
    fn i32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i32x4_lt(a, b)
    }

    #[inline(always)]
    fn u8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_max(a, b)
    }

    #[inline(always)]
    fn u16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u16x8_max(a, b)
    }

    #[inline(always)]
    fn u32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u32x4_max(a, b)
    }

    #[inline(always)]
    fn i8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i8x16_max(a, b)
    }

    #[inline(always)]
    fn i16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i16x8_max(a, b)
    }

    #[inline(always)]
    fn i32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i32x4_max(a, b)
    }

    #[inline(always)]
    fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_min(a, b)
    }

    #[inline(always)]
    fn u16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u16x8_min(a, b)
    }

    #[inline(always)]
    fn u32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u32x4_min(a, b)
    }

    #[inline(always)]
    fn i8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i8x16_min(a, b)
    }

    #[inline(always)]
    fn i16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i16x8_min(a, b)
    }

    #[inline(always)]
    fn i32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i32x4_min(a, b)
    }

    #[inline(always)]
    fn u16x8_bswap(self, a: Self::V128) -> Self::V128 {
        self.u8x16_swizzle(a, self.load(crate::common::bswap::SHUFFLE_U16X8))
    }

    #[inline(always)]
    fn u32x4_bswap(self, a: Self::V128) -> Self::V128 {
        self.u8x16_swizzle(a, self.load(crate::common::bswap::SHUFFLE_U32X4))
    }

    #[inline(always)]
    fn u64x2_bswap(self, a: Self::V128) -> Self::V128 {
        self.u8x16_swizzle(a, self.load(crate::common::bswap::SHUFFLE_U64X2))
    }

    #[inline(always)]
    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        u8x16_swizzle(a, b)
    }
}

unsafe impl isa::SIMD256 for SIMD128 {
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

    mock256_instructions!();
}
