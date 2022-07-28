use super::InstructionSet;

pub unsafe trait SIMD128: InstructionSet {
    type V128: Copy;

    unsafe fn v128_load(self, addr: *const u8) -> Self::V128;
    unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128;
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128);

    fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn v128_to_bytes(self, a: Self::V128) -> [u8; 16];
    fn v128_create_zero(self) -> Self::V128;
    fn v128_all_zero(self, a: Self::V128) -> bool;
    fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u8x16_splat(self, x: u8) -> Self::V128;
    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u8x16_any_zero(self, a: Self::V128) -> bool;
    fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn i8x16_splat(self, x: i8) -> Self::V128;
    fn i8x16_cmp_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i8x16_cmp_eq(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u16x8_splat(self, x: u16) -> Self::V128;

    fn u32x4_splat(self, x: u32) -> Self::V128;
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_cmp_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn i32x4_cmp_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;
}
