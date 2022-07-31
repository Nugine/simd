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
    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u8x16_any_zero(self, a: Self::V128) -> bool;

    fn i8x16_splat(self, x: i8) -> Self::V128;
    fn i8x16_eq(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u16x8_splat(self, x: u16) -> Self::V128;

    fn u32x4_splat(self, x: u32) -> Self::V128;
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;

    // ----refactor----

    fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_add(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u64x2_add(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u64x2_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn i8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i16x8_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u16x8_bswap(self, a: Self::V128) -> Self::V128;
    fn u32x4_bswap(self, a: Self::V128) -> Self::V128;
    fn u64x2_bswap(self, a: Self::V128) -> Self::V128;
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! inherit_simd128 {
    ($self: ty, $super: ty, $upcast:ident) => {
        unsafe impl SIMD128 for $self {
            type V128 = <$super as SIMD128>::V128;

            #[inline(always)]
            unsafe fn v128_load(self, addr: *const u8) -> Self::V128 {
                self.$upcast().v128_load(addr)
            }

            #[inline(always)]
            unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128 {
                self.$upcast().v128_load_unaligned(addr)
            }

            #[inline(always)]
            unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
                self.$upcast().v128_store_unaligned(addr, a)
            }

            #[inline(always)]
            fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().v128_or(a, b)
            }

            #[inline(always)]
            fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().v128_and(a, b)
            }

            #[inline(always)]
            fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
                self.$upcast().v128_to_bytes(a)
            }

            #[inline(always)]
            fn v128_create_zero(self) -> Self::V128 {
                self.$upcast().v128_create_zero()
            }

            #[inline(always)]
            fn v128_all_zero(self, a: Self::V128) -> bool {
                self.$upcast().v128_all_zero(a)
            }

            #[inline(always)]
            fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().v128_andnot(a, b)
            }

            #[inline(always)]
            fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().v128_xor(a, b)
            }

            #[inline(always)]
            fn u8x16_splat(self, x: u8) -> Self::V128 {
                self.$upcast().u8x16_splat(x)
            }

            #[inline(always)]
            fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u8x16_swizzle(a, b)
            }

            #[inline(always)]
            fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u8x16_sub_sat(a, b)
            }

            #[inline(always)]
            fn u8x16_any_zero(self, a: Self::V128) -> bool {
                self.$upcast().u8x16_any_zero(a)
            }

            #[inline(always)]
            fn i8x16_splat(self, x: i8) -> Self::V128 {
                self.$upcast().i8x16_splat(x)
            }

            #[inline(always)]
            fn i8x16_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i8x16_eq(a, b)
            }

            #[inline(always)]
            fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                self.$upcast().u16x8_shl::<IMM8>(a)
            }

            #[inline(always)]
            fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                self.$upcast().u16x8_shr::<IMM8>(a)
            }

            #[inline(always)]
            fn u16x8_splat(self, x: u16) -> Self::V128 {
                self.$upcast().u16x8_splat(x)
            }

            #[inline(always)]
            fn u32x4_splat(self, x: u32) -> Self::V128 {
                self.$upcast().u32x4_splat(x)
            }

            #[inline(always)]
            fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                self.$upcast().u32x4_shl::<IMM8>(a)
            }

            #[inline(always)]
            fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                self.$upcast().u32x4_shr::<IMM8>(a)
            }

            #[inline(always)]
            fn u32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u32x4_lt(a, b)
            }

            #[inline(always)]
            fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u8x16_add(a, b)
            }

            #[inline(always)]
            fn u16x8_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u16x8_add(a, b)
            }

            #[inline(always)]
            fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u32x4_add(a, b)
            }

            #[inline(always)]
            fn u64x2_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u64x2_add(a, b)
            }

            #[inline(always)]
            fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u8x16_sub(a, b)
            }

            #[inline(always)]
            fn u16x8_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u16x8_sub(a, b)
            }

            #[inline(always)]
            fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u32x4_sub(a, b)
            }

            #[inline(always)]
            fn u64x2_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u64x2_sub(a, b)
            }

            #[inline(always)]
            fn i8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i8x16_lt(a, b)
            }

            #[inline(always)]
            fn i16x8_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i16x8_lt(a, b)
            }

            #[inline(always)]
            fn i32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i32x4_lt(a, b)
            }

            #[inline(always)]
            fn u8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u8x16_max(a, b)
            }

            #[inline(always)]
            fn u16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u16x8_max(a, b)
            }

            #[inline(always)]
            fn u32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u32x4_max(a, b)
            }

            #[inline(always)]
            fn i8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i8x16_max(a, b)
            }

            #[inline(always)]
            fn i16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i16x8_max(a, b)
            }

            #[inline(always)]
            fn i32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i32x4_max(a, b)
            }

            #[inline(always)]
            fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u8x16_min(a, b)
            }

            #[inline(always)]
            fn u16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u16x8_min(a, b)
            }

            #[inline(always)]
            fn u32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().u32x4_min(a, b)
            }

            #[inline(always)]
            fn i8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i8x16_min(a, b)
            }

            #[inline(always)]
            fn i16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i16x8_min(a, b)
            }

            #[inline(always)]
            fn i32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                self.$upcast().i32x4_min(a, b)
            }

            #[inline(always)]
            fn u16x8_bswap(self, a: Self::V128) -> Self::V128 {
                self.$upcast().u16x8_bswap(a)
            }

            #[inline(always)]
            fn u32x4_bswap(self, a: Self::V128) -> Self::V128 {
                self.$upcast().u32x4_bswap(a)
            }

            #[inline(always)]
            fn u64x2_bswap(self, a: Self::V128) -> Self::V128 {
                self.$upcast().u64x2_bswap(a)
            }
        }
    };
}
