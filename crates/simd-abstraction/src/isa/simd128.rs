use super::InstructionSet;

pub unsafe trait SIMD128: InstructionSet {
    type V128: Copy;

    fn v128_to_bytes(self, a: Self::V128) -> [u8; 16];
    fn v128_create_zero(self) -> Self::V128;
    fn v128_all_zero(self, a: Self::V128) -> bool;

    // ----refactor----

    unsafe fn v128_load(self, addr: *const u8) -> Self::V128;
    unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128;
    unsafe fn v128_store(self, addr: *mut u8, a: Self::V128);
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128);

    fn v128_not(self, a: Self::V128) -> Self::V128;
    fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u8x16_splat(self, x: u8) -> Self::V128;
    fn u16x8_splat(self, x: u16) -> Self::V128;
    fn u32x4_splat(self, x: u32) -> Self::V128;
    fn u64x2_splat(self, x: u64) -> Self::V128;
    fn i8x16_splat(self, x: i8) -> Self::V128;
    fn i16x8_splat(self, x: i16) -> Self::V128;
    fn i32x4_splat(self, x: i32) -> Self::V128;
    fn i64x2_splat(self, x: i64) -> Self::V128;

    fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_add(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u64x2_add(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u64x2_sub(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn i16x8_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128;

    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128;
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128;

    fn u8x16_eq(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_eq(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_eq(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u16x8_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;
    fn u32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128;
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

    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128;

    fn u8x16_any_zero(self, a: Self::V128) -> bool;
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! inherit_simd128 {
    ($self: ty, $super: ty, $upcast:ident) => {
        unsafe impl SIMD128 for $self {
            type V128 = <$super as SIMD128>::V128;

            #[inline(always)]
            fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
                <$super as SIMD128>::v128_to_bytes(self.$upcast(), a)
            }

            #[inline(always)]
            fn v128_create_zero(self) -> Self::V128 {
                self.$upcast().v128_create_zero()
            }

            #[inline(always)]
            fn v128_all_zero(self, a: Self::V128) -> bool {
                <$super as SIMD128>::v128_all_zero(self.$upcast(), a)
            }

            #[inline(always)]
            unsafe fn v128_load(self, addr: *const u8) -> Self::V128 {
                <$super as SIMD128>::v128_load(self.$upcast(), addr)
            }

            #[inline(always)]
            unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128 {
                <$super as SIMD128>::v128_load_unaligned(self.$upcast(), addr)
            }

            #[inline(always)]
            unsafe fn v128_store(self, addr: *mut u8, a: Self::V128) {
                <$super as SIMD128>::v128_store(self.$upcast(), addr, a)
            }

            #[inline(always)]
            unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
                <$super as SIMD128>::v128_store_unaligned(self.$upcast(), addr, a)
            }

            #[inline(always)]
            fn v128_not(self, a: Self::V128) -> Self::V128 {
                <$super as SIMD128>::v128_not(self.$upcast(), a)
            }

            #[inline(always)]
            fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::v128_and(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::v128_or(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::v128_xor(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::v128_andnot(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x16_splat(self, x: u8) -> Self::V128 {
                <$super as SIMD128>::u8x16_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u16x8_splat(self, x: u16) -> Self::V128 {
                <$super as SIMD128>::u16x8_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u32x4_splat(self, x: u32) -> Self::V128 {
                <$super as SIMD128>::u32x4_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u64x2_splat(self, x: u64) -> Self::V128 {
                <$super as SIMD128>::u64x2_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i8x16_splat(self, x: i8) -> Self::V128 {
                <$super as SIMD128>::i8x16_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i16x8_splat(self, x: i16) -> Self::V128 {
                <$super as SIMD128>::i16x8_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i32x4_splat(self, x: i32) -> Self::V128 {
                <$super as SIMD128>::i32x4_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i64x2_splat(self, x: i64) -> Self::V128 {
                <$super as SIMD128>::i64x2_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u8x16_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x8_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u32x4_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u64x2_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u64x2_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u8x16_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x8_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u32x4_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u64x2_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u64x2_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u8x16_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x8_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i8x16_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x8_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i16x8_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_shl::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u32x4_shl::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_shr::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u32x4_shr::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u8x16_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u8x16_eq(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x8_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_eq(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x4_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u32x4_eq(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u8x16_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x8_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u32x4_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i8x16_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x8_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i16x8_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i32x4_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u8x16_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u32x4_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x16_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i8x16_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x8_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i16x8_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i32x4_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u8x16_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u32x4_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i8x16_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x8_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i16x8_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x4_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::i32x4_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x8_bswap(self, a: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u16x8_bswap(self.$upcast(), a)
            }

            #[inline(always)]
            fn u32x4_bswap(self, a: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u32x4_bswap(self.$upcast(), a)
            }

            #[inline(always)]
            fn u64x2_bswap(self, a: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u64x2_bswap(self.$upcast(), a)
            }

            #[inline(always)]
            fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
                <$super as SIMD128>::u8x16_swizzle(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x16_any_zero(self, a: Self::V128) -> bool {
                <$super as SIMD128>::u8x16_any_zero(self.$upcast(), a)
            }
        }
    };
}
