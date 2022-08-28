use crate::isa::SimdLoad;
use crate::isa::SIMD256 as _;
use crate::isa::{self, InstructionSet};

#[cfg(target_arch = "wasm32")]
use core::arch::wasm32::*;

define_isa!(SIMD128, "simd128");

unsafe impl isa::SIMD128 for SIMD128 {
    type V128 = v128;

    #[inline(always)]
    fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
        unsafe { core::mem::transmute(a) }
    }

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
    unsafe fn v128_store(self, addr: *mut u8, a: Self::V128) {
        debug_assert_ptr_align!(addr, 16);
        v128_store(addr.cast(), a)
    }

    #[inline(always)]
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
        v128_store(addr.cast(), a)
    }

    #[inline(always)]
    fn v128_create_zero(self) -> Self::V128 {
        u8x16_splat(0)
    }

    #[inline(always)]
    fn v128_not(self, a: Self::V128) -> Self::V128 {
        v128_not(a)
    }

    #[inline(always)]
    fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        v128_and(a, b)
    }

    #[inline(always)]
    fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        v128_or(a, b)
    }

    #[inline(always)]
    fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        v128_xor(a, b)
    }

    #[inline(always)]
    fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        v128_andnot(a, b)
    }

    #[inline(always)]
    fn v128_all_zero(self, a: Self::V128) -> bool {
        !v128_any_true(a)
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
    fn i16x8_mul_lo(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i16x8_mul(a, b)
    }

    #[inline(always)]
    fn i32x4_mul_lo(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        i32x4_mul(a, b)
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

    #[inline(always)]
    fn u8x16_any_zero(self, a: Self::V128) -> bool {
        !u8x16_all_true(a)
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
}

unsafe impl isa::SIMD512 for SIMD128 {
    type V512 = (v128, v128, v128, v128);

    #[inline(always)]
    fn v512_from_v256x2(self, a: Self::V256, b: Self::V256) -> Self::V512 {
        (a.0, a.1, b.0, b.1)
    }

    #[inline(always)]
    fn v512_to_v256x2(self, a: Self::V512) -> (Self::V256, Self::V256) {
        ((a.0, a.1), (a.2, a.3))
    }

    #[inline(always)]
    fn v512_to_bytes(self, a: Self::V512) -> [u8; 64] {
        unsafe { core::mem::transmute([a.0, a.1, a.2, a.3]) }
    }
}

impl SIMD128 {
    /// for each bit: if a == 1 { b } else { c }
    ///
    /// ans = ((b ^ c) & a) ^ c
    #[inline(always)]
    #[must_use]
    pub fn v256_bsl(
        self,
        a: <Self as isa::SIMD256>::V256,
        b: <Self as isa::SIMD256>::V256,
        c: <Self as isa::SIMD256>::V256,
    ) -> <Self as isa::SIMD256>::V256 {
        self.v256_xor(self.v256_and(self.v256_xor(b, c), a), c)
    }

    #[inline(always)]
    #[must_use]
    pub fn u8x16_bitmask(self, a: v128) -> u16 {
        i8x16_bitmask(a)
    }

    #[inline(always)]
    #[must_use]
    pub fn u8x32_bitmask(self, a: <Self as isa::SIMD256>::V256) -> u32 {
        let m0 = self.u8x16_bitmask(a.0) as u32;
        let m1 = self.u8x16_bitmask(a.1) as u32;
        (m1 << 16) | m0
    }
}
