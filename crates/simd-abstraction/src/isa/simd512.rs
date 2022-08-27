use super::mock::*;
use super::SIMD256;

pub unsafe trait SIMD512: SIMD256 {
    type V512: Copy;

    fn v512_from_v256x2(self, a: Self::V256, b: Self::V256) -> Self::V512;
    fn v512_to_v256x2(self, a: Self::V512) -> (Self::V256, Self::V256);
    fn v512_to_bytes(self, a: Self::V512) -> [u8; 64];

    #[inline(always)]
    unsafe fn v512_load(self, addr: *const u8) -> Self::V512 {
        debug_assert_ptr_align!(addr, 32);
        let a0 = self.v256_load(addr);
        let a1 = self.v256_load(addr.add(16));
        self.v512_from_v256x2(a0, a1)
    }

    #[inline(always)]
    unsafe fn v512_load_unaligned(self, addr: *const u8) -> Self::V512 {
        let a0 = self.v256_load_unaligned(addr);
        let a1 = self.v256_load_unaligned(addr.add(16));
        self.v512_from_v256x2(a0, a1)
    }

    #[inline(always)]
    unsafe fn v512_store(self, addr: *mut u8, a: Self::V512) {
        debug_assert_ptr_align!(addr, 32);
        let (a0, a1) = self.v512_to_v256x2(a);
        self.v256_store(addr, a0);
        self.v256_store(addr.add(16), a1);
    }

    #[inline(always)]
    unsafe fn v512_store_unaligned(self, addr: *mut u8, a: Self::V512) {
        let (a0, a1) = self.v512_to_v256x2(a);
        self.v256_store_unaligned(addr, a0);
        self.v256_store_unaligned(addr.add(16), a1);
    }

    #[inline(always)]
    fn v512_create_zero(self) -> Self::V512 {
        self.v512_from_v256x2(self.v256_create_zero(), self.v256_create_zero())
    }

    #[inline(always)]
    fn v512_get_low(self, a: Self::V512) -> Self::V256 {
        self.v512_to_v256x2(a).0
    }

    #[inline(always)]
    fn v512_get_high(self, a: Self::V512) -> Self::V256 {
        self.v512_to_v256x2(a).1
    }

    #[inline(always)]
    fn v512_not(self, a: Self::V512) -> Self::V512 {
        simd512_vop1(self, a, Self::v256_not)
    }

    #[inline(always)]
    fn v512_and(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::v256_and)
    }

    #[inline(always)]
    fn v512_or(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::v256_or)
    }

    #[inline(always)]
    fn v512_xor(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::v256_xor)
    }

    #[inline(always)]
    fn v512_andnot(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::v256_andnot)
    }

    #[inline(always)]
    fn v512_all_zero(self, a: Self::V512) -> bool {
        let a = self.v512_to_v256x2(a);
        self.v256_all_zero(self.v256_or(a.0, a.1))
    }

    #[inline(always)]
    fn u8x64_splat(self, x: u8) -> Self::V512 {
        simd512_double(self, || self.u8x32_splat(x))
    }

    #[inline(always)]
    fn u16x32_splat(self, x: u16) -> Self::V512 {
        simd512_double(self, || self.u16x16_splat(x))
    }

    #[inline(always)]
    fn u32x16_splat(self, x: u32) -> Self::V512 {
        simd512_double(self, || self.u32x8_splat(x))
    }

    #[inline(always)]
    fn u64x8_splat(self, x: u64) -> Self::V512 {
        simd512_double(self, || self.u64x4_splat(x))
    }

    #[inline(always)]
    fn i8x64_splat(self, x: i8) -> Self::V512 {
        simd512_double(self, || self.i8x32_splat(x))
    }

    #[inline(always)]
    fn i16x32_splat(self, x: i16) -> Self::V512 {
        simd512_double(self, || self.i16x16_splat(x))
    }

    #[inline(always)]
    fn i32x16_splat(self, x: i32) -> Self::V512 {
        simd512_double(self, || self.i32x8_splat(x))
    }

    #[inline(always)]
    fn i64x8_splat(self, x: i64) -> Self::V512 {
        simd512_double(self, || self.i64x4_splat(x))
    }

    #[inline(always)]
    fn u8x64_add(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u8x32_add)
    }

    #[inline(always)]
    fn u16x32_add(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u16x16_add)
    }

    #[inline(always)]
    fn u32x16_add(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u32x8_add)
    }

    #[inline(always)]
    fn u64x8_add(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u64x4_add)
    }

    #[inline(always)]
    fn u8x64_sub(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u8x32_sub)
    }

    #[inline(always)]
    fn u16x32_sub(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u16x16_sub)
    }

    #[inline(always)]
    fn u32x16_sub(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u32x8_sub)
    }

    #[inline(always)]
    fn u64x8_sub(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u64x4_sub)
    }

    #[inline(always)]
    fn u8x64_sub_sat(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u8x32_sub_sat)
    }

    #[inline(always)]
    fn u16x32_sub_sat(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u16x16_sub_sat)
    }

    #[inline(always)]
    fn i8x64_sub_sat(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i8x32_sub_sat)
    }

    #[inline(always)]
    fn i16x32_sub_sat(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i16x16_sub_sat)
    }

    #[inline(always)]
    fn i16x32_mul_lo(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i16x16_mul_lo)
    }

    #[inline(always)]
    fn i32x16_mul_lo(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i32x8_mul_lo)
    }

    #[inline(always)]
    fn u16x32_shl<const IMM8: i32>(self, a: Self::V512) -> Self::V512 {
        simd512_vop1(self, a, Self::u16x16_shl::<IMM8>)
    }

    #[inline(always)]
    fn u32x16_shl<const IMM8: i32>(self, a: Self::V512) -> Self::V512 {
        simd512_vop1(self, a, Self::u32x8_shl::<IMM8>)
    }

    #[inline(always)]
    fn u16x32_shr<const IMM8: i32>(self, a: Self::V512) -> Self::V512 {
        simd512_vop1(self, a, Self::u16x16_shr::<IMM8>)
    }

    #[inline(always)]
    fn u32x16_shr<const IMM8: i32>(self, a: Self::V512) -> Self::V512 {
        simd512_vop1(self, a, Self::u32x8_shr::<IMM8>)
    }

    #[inline(always)]
    fn u8x64_eq(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u8x32_eq)
    }

    #[inline(always)]
    fn u16x32_eq(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u16x16_eq)
    }

    #[inline(always)]
    fn u32x16_eq(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u32x8_eq)
    }

    #[inline(always)]
    fn u8x64_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u8x32_lt)
    }

    #[inline(always)]
    fn u16x32_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u16x16_lt)
    }

    #[inline(always)]
    fn u32x16_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u32x8_lt)
    }

    #[inline(always)]
    fn i8x64_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i8x32_lt)
    }

    #[inline(always)]
    fn i16x32_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i16x16_lt)
    }

    #[inline(always)]
    fn i32x16_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i32x8_lt)
    }

    #[inline(always)]
    fn u8x64_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u8x32_max)
    }

    #[inline(always)]
    fn u16x32_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u16x16_max)
    }

    #[inline(always)]
    fn u32x16_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u32x8_max)
    }

    #[inline(always)]
    fn i8x64_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i8x32_max)
    }

    #[inline(always)]
    fn i16x32_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i16x16_max)
    }

    #[inline(always)]
    fn i32x16_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i32x8_max)
    }

    #[inline(always)]
    fn u8x64_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u8x32_min)
    }

    #[inline(always)]
    fn u16x32_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u16x16_min)
    }

    #[inline(always)]
    fn u32x16_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u32x8_min)
    }

    #[inline(always)]
    fn i8x64_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i8x32_min)
    }

    #[inline(always)]
    fn i16x32_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i16x16_min)
    }

    #[inline(always)]
    fn i32x16_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::i32x8_min)
    }

    #[inline(always)]
    fn u16x32_bswap(self, a: Self::V512) -> Self::V512 {
        simd512_vop1(self, a, Self::u16x16_bswap)
    }

    #[inline(always)]
    fn u32x16_bswap(self, a: Self::V512) -> Self::V512 {
        simd512_vop1(self, a, Self::u32x8_bswap)
    }

    #[inline(always)]
    fn u64x8_bswap(self, a: Self::V512) -> Self::V512 {
        simd512_vop1(self, a, Self::u64x4_bswap)
    }

    #[inline(always)]
    fn u8x16x4_swizzle(self, a: Self::V512, b: Self::V512) -> Self::V512 {
        simd512_vop2(self, a, b, Self::u8x16x2_swizzle)
    }

    #[inline(always)]
    fn u8x64_any_zero(self, a: Self::V512) -> bool {
        let a = self.v512_to_v256x2(a);
        self.u8x32_any_zero(self.u8x32_min(a.0, a.1))
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! inherit_simd512 {
    ($self: ty, $super: ty, $upcast: ident) => {
        unsafe impl SIMD512 for $self {
            type V512 = <$super as SIMD512>::V512;

            #[inline(always)]
            fn v512_from_v256x2(self, a: Self::V256, b: Self::V256) -> Self::V512 {
                <$super as SIMD512>::v512_from_v256x2(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn v512_to_v256x2(self, a: Self::V512) -> (Self::V256, Self::V256) {
                <$super as SIMD512>::v512_to_v256x2(self.$upcast(), a)
            }

            #[inline(always)]
            fn v512_to_bytes(self, a: Self::V512) -> [u8; 64] {
                <$super as SIMD512>::v512_to_bytes(self.$upcast(), a)
            }

            #[inline(always)]
            unsafe fn v512_load(self, addr: *const u8) -> Self::V512 {
                <$super as SIMD512>::v512_load(self.$upcast(), addr)
            }

            #[inline(always)]
            unsafe fn v512_load_unaligned(self, addr: *const u8) -> Self::V512 {
                <$super as SIMD512>::v512_load_unaligned(self.$upcast(), addr)
            }

            #[inline(always)]
            unsafe fn v512_store(self, addr: *mut u8, a: Self::V512) {
                <$super as SIMD512>::v512_store(self.$upcast(), addr, a)
            }

            #[inline(always)]
            unsafe fn v512_store_unaligned(self, addr: *mut u8, a: Self::V512) {
                <$super as SIMD512>::v512_store_unaligned(self.$upcast(), addr, a)
            }

            #[inline(always)]
            fn u8x64_splat(self, x: u8) -> Self::V512 {
                <$super as SIMD512>::u8x64_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u16x32_splat(self, x: u16) -> Self::V512 {
                <$super as SIMD512>::u16x32_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u32x16_splat(self, x: u32) -> Self::V512 {
                <$super as SIMD512>::u32x16_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u64x8_splat(self, x: u64) -> Self::V512 {
                <$super as SIMD512>::u64x8_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i8x64_splat(self, x: i8) -> Self::V512 {
                <$super as SIMD512>::i8x64_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i16x32_splat(self, x: i16) -> Self::V512 {
                <$super as SIMD512>::i16x32_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i32x16_splat(self, x: i32) -> Self::V512 {
                <$super as SIMD512>::i32x16_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i64x8_splat(self, x: i64) -> Self::V512 {
                <$super as SIMD512>::i64x8_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u8x64_add(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u8x64_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x32_add(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x16_add(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u32x16_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u64x8_add(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u64x8_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x64_sub(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u8x64_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x32_sub(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x16_sub(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u32x16_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x64_sub_sat(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u8x64_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x32_sub_sat(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x64_sub_sat(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i8x64_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x32_sub_sat(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i16x32_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x32_mul_lo(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i16x32_mul_lo(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x16_mul_lo(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i32x16_mul_lo(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u64x8_sub(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u64x8_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x32_shl<const IMM8: i32>(self, a: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_shl::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u32x16_shl<const IMM8: i32>(self, a: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u32x16_shl::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u16x32_shr<const IMM8: i32>(self, a: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_shr::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u32x16_shr<const IMM8: i32>(self, a: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u32x16_shr::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u8x64_eq(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u8x64_eq(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x32_eq(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_eq(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x16_eq(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u32x16_eq(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x64_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u8x64_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x32_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x16_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u32x16_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x64_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i8x64_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x32_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i16x32_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x16_lt(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i32x16_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x64_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u8x64_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x32_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x16_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u32x16_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x64_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i8x64_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x32_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i16x32_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x16_max(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i32x16_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x64_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u8x64_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x32_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x16_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u32x16_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x64_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i8x64_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x32_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i16x32_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x16_min(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::i32x16_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x16x4_swizzle(self, a: Self::V512, b: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u8x16x4_swizzle(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x32_bswap(self, a: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u16x32_bswap(self.$upcast(), a)
            }

            #[inline(always)]
            fn u32x16_bswap(self, a: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u32x16_bswap(self.$upcast(), a)
            }

            #[inline(always)]
            fn u64x8_bswap(self, a: Self::V512) -> Self::V512 {
                <$super as SIMD512>::u64x8_bswap(self.$upcast(), a)
            }
        }
    };
}
