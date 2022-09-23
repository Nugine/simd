use crate::mask::*;
use crate::{InstructionSet, SIMD128, SIMD256, V128, V256};

pub unsafe trait Scalable<V: Copy>: InstructionSet {
    fn and(self, a: V, b: V) -> V;
    fn or(self, a: V, b: V) -> V;
    fn xor(self, a: V, b: V) -> V;

    fn u8xn_splat(self, x: u8) -> V;
    fn i8xn_splat(self, x: i8) -> V;

    fn u8xn_add(self, a: V, b: V) -> V;

    fn u8xn_sub(self, a: V, b: V) -> V;

    fn u8xn_add_sat(self, a: V, b: V) -> V;
    fn i8xn_add_sat(self, a: V, b: V) -> V;

    fn u8xn_sub_sat(self, a: V, b: V) -> V;

    fn i8xn_lt(self, a: V, b: V) -> V;

    fn u16xn_shl<const IMM8: i32>(self, a: V) -> V;

    fn u16xn_shr<const IMM8: i32>(self, a: V) -> V;
    fn u32xn_shr<const IMM8: i32>(self, a: V) -> V;

    fn u8xn_avgr(self, a: V, b: V) -> V;

    fn u8x16xn_swizzle(self, a: V, b: V) -> V;

    fn mask8xn_all(self, a: V) -> bool;
    fn mask8xn_any(self, a: V) -> bool;

    fn u8xn_highbit_all(self, a: V) -> bool;
    fn u8xn_highbit_any(self, a: V) -> bool;
}

unsafe impl<S> Scalable<V128> for S
where
    S: SIMD128,
{
    #[inline(always)]
    fn and(self, a: V128, b: V128) -> V128 {
        self.v128_and(a, b)
    }

    #[inline(always)]
    fn or(self, a: V128, b: V128) -> V128 {
        self.v128_or(a, b)
    }

    #[inline(always)]
    fn xor(self, a: V128, b: V128) -> V128 {
        self.v128_xor(a, b)
    }

    #[inline(always)]
    fn u8xn_splat(self, x: u8) -> V128 {
        self.u8x16_splat(x)
    }

    #[inline(always)]
    fn i8xn_splat(self, x: i8) -> V128 {
        self.i8x16_splat(x)
    }

    #[inline(always)]
    fn u8xn_add(self, a: V128, b: V128) -> V128 {
        self.u8x16_add(a, b)
    }

    #[inline(always)]
    fn u8xn_sub(self, a: V128, b: V128) -> V128 {
        self.u8x16_sub(a, b)
    }

    #[inline(always)]
    fn u8xn_add_sat(self, a: V128, b: V128) -> V128 {
        self.u8x16_add_sat(a, b)
    }

    #[inline(always)]
    fn i8xn_add_sat(self, a: V128, b: V128) -> V128 {
        self.i8x16_add_sat(a, b)
    }

    #[inline(always)]
    fn u8xn_sub_sat(self, a: V128, b: V128) -> V128 {
        self.u8x16_sub_sat(a, b)
    }

    #[inline(always)]
    fn i8xn_lt(self, a: V128, b: V128) -> V128 {
        self.i8x16_lt(a, b)
    }

    #[inline(always)]
    fn u16xn_shl<const IMM8: i32>(self, a: V128) -> V128 {
        self.u16x8_shl::<IMM8>(a)
    }

    #[inline(always)]
    fn u16xn_shr<const IMM8: i32>(self, a: V128) -> V128 {
        self.u16x8_shr::<IMM8>(a)
    }

    #[inline(always)]
    fn u32xn_shr<const IMM8: i32>(self, a: V128) -> V128 {
        self.u32x4_shr::<IMM8>(a)
    }

    #[inline(always)]
    fn u8xn_avgr(self, a: V128, b: V128) -> V128 {
        self.u8x16_avgr(a, b)
    }

    #[inline(always)]
    fn u8x16xn_swizzle(self, a: V128, b: V128) -> V128 {
        self.u8x16_swizzle(a, b)
    }

    #[inline(always)]
    fn mask8xn_all(self, a: V128) -> bool {
        mask8x16_all(self, a)
    }

    #[inline(always)]
    fn mask8xn_any(self, a: V128) -> bool {
        mask8x16_any(self, a)
    }

    #[inline(always)]
    fn u8xn_highbit_all(self, a: V128) -> bool {
        u8x16_highbit_all(self, a)
    }

    #[inline(always)]
    fn u8xn_highbit_any(self, a: V128) -> bool {
        u8x16_highbit_any(self, a)
    }
}

unsafe impl<S> Scalable<V256> for S
where
    S: SIMD256,
{
    #[inline(always)]
    fn and(self, a: V256, b: V256) -> V256 {
        self.v256_and(a, b)
    }

    #[inline(always)]
    fn or(self, a: V256, b: V256) -> V256 {
        self.v256_or(a, b)
    }

    #[inline(always)]
    fn xor(self, a: V256, b: V256) -> V256 {
        self.v256_xor(a, b)
    }

    #[inline(always)]
    fn u8xn_splat(self, x: u8) -> V256 {
        self.u8x32_splat(x)
    }

    #[inline(always)]
    fn i8xn_splat(self, x: i8) -> V256 {
        self.i8x32_splat(x)
    }

    #[inline(always)]
    fn u8xn_add(self, a: V256, b: V256) -> V256 {
        self.u8x32_add(a, b)
    }

    #[inline(always)]
    fn u8xn_sub(self, a: V256, b: V256) -> V256 {
        self.u8x32_sub(a, b)
    }

    #[inline(always)]
    fn u8xn_add_sat(self, a: V256, b: V256) -> V256 {
        self.u8x32_add_sat(a, b)
    }

    #[inline(always)]
    fn i8xn_add_sat(self, a: V256, b: V256) -> V256 {
        self.i8x32_add_sat(a, b)
    }

    #[inline(always)]
    fn u8xn_sub_sat(self, a: V256, b: V256) -> V256 {
        self.u8x32_sub_sat(a, b)
    }

    #[inline(always)]
    fn i8xn_lt(self, a: V256, b: V256) -> V256 {
        self.i8x32_lt(a, b)
    }

    #[inline(always)]
    fn u16xn_shl<const IMM8: i32>(self, a: V256) -> V256 {
        self.u16x16_shl::<IMM8>(a)
    }

    #[inline(always)]
    fn u16xn_shr<const IMM8: i32>(self, a: V256) -> V256 {
        self.u16x16_shr::<IMM8>(a)
    }

    #[inline(always)]
    fn u32xn_shr<const IMM8: i32>(self, a: V256) -> V256 {
        self.u32x8_shr::<IMM8>(a)
    }

    #[inline(always)]
    fn u8xn_avgr(self, a: V256, b: V256) -> V256 {
        self.u8x32_avgr(a, b)
    }

    #[inline(always)]
    fn u8x16xn_swizzle(self, a: V256, b: V256) -> V256 {
        self.u8x16x2_swizzle(a, b)
    }

    #[inline(always)]
    fn mask8xn_all(self, a: V256) -> bool {
        mask8x32_all(self, a)
    }

    #[inline(always)]
    fn mask8xn_any(self, a: V256) -> bool {
        mask8x32_any(self, a)
    }

    #[inline(always)]
    fn u8xn_highbit_all(self, a: V256) -> bool {
        u8x32_highbit_all(self, a)
    }

    #[inline(always)]
    fn u8xn_highbit_any(self, a: V256) -> bool {
        u8x32_highbit_any(self, a)
    }
}
