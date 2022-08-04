use super::SIMD128;

#[inline(always)]
fn vmap<S: SIMD256>(s: S, a: S::V256, f: impl Fn(S, S::V128) -> S::V128) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = (f(s, a.0), f(s, a.1));
    s.v256_from_v128x2(b.0, b.1)
}

#[inline(always)]
fn vmerge<S: SIMD256>(s: S, a: S::V256, b: S::V256, f: impl Fn(S, S::V128, S::V128) -> S::V128) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = s.v256_to_v128x2(b);
    let c = (f(s, a.0, b.0), f(s, a.1, b.1));
    s.v256_from_v128x2(c.0, c.1)
}

#[inline(always)]
fn double<S: SIMD256>(s: S, f: impl FnOnce() -> S::V128) -> S::V256 {
    let a = f();
    s.v256_from_v128x2(a, a)
}

pub unsafe trait SIMD256: SIMD128 {
    type V256: Copy;

    fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256;
    fn v256_to_v128x2(self, a: Self::V256) -> (Self::V128, Self::V128);
    fn v256_to_bytes(self, a: Self::V256) -> [u8; 32];

    #[inline(always)]
    fn v256_create_zero(self) -> Self::V256 {
        self.v256_from_v128x2(self.v128_create_zero(), self.v128_create_zero())
    }

    #[inline(always)]
    fn v256_all_zero(self, a: Self::V256) -> bool {
        let a = self.v256_to_v128x2(a);
        self.v128_all_zero(self.v128_or(a.0, a.1))
    }

    #[inline(always)]
    fn v256_get_low(self, a: Self::V256) -> Self::V128 {
        self.v256_to_v128x2(a).0
    }

    #[inline(always)]
    fn v256_get_high(self, a: Self::V256) -> Self::V128 {
        self.v256_to_v128x2(a).1
    }

    #[inline(always)]
    fn u8x32_any_zero(self, a: Self::V256) -> bool {
        let a = self.v256_to_v128x2(a);
        self.u8x16_any_zero(self.u8x16_min(a.0, a.1))
    }

    // ----refactor----

    #[inline(always)]
    unsafe fn v256_load(self, addr: *const u8) -> Self::V256 {
        debug_assert_ptr_align!(addr, 32);
        let a0 = self.v128_load(addr);
        let a1 = self.v128_load(addr.add(16));
        self.v256_from_v128x2(a0, a1)
    }

    #[inline(always)]
    unsafe fn v256_load_unaligned(self, addr: *const u8) -> Self::V256 {
        let a0 = self.v128_load_unaligned(addr);
        let a1 = self.v128_load_unaligned(addr.add(16));
        self.v256_from_v128x2(a0, a1)
    }

    #[inline(always)]
    unsafe fn v256_store(self, addr: *mut u8, a: Self::V256) {
        debug_assert_ptr_align!(addr, 32);
        let (a0, a1) = self.v256_to_v128x2(a);
        self.v128_store(addr, a0);
        self.v128_store(addr.add(16), a1);
    }

    #[inline(always)]
    unsafe fn v256_store_unaligned(self, addr: *mut u8, a: Self::V256) {
        let (a0, a1) = self.v256_to_v128x2(a);
        self.v128_store_unaligned(addr, a0);
        self.v128_store_unaligned(addr.add(16), a1);
    }

    #[inline(always)]
    fn v256_not(self, a: Self::V256) -> Self::V256 {
        vmap(self, a, Self::v128_not)
    }

    #[inline(always)]
    fn v256_and(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::v128_and)
    }

    #[inline(always)]
    fn v256_or(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::v128_or)
    }

    #[inline(always)]
    fn v256_xor(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::v128_xor)
    }

    #[inline(always)]
    fn v256_andnot(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::v128_andnot)
    }

    #[inline(always)]
    fn u8x32_splat(self, x: u8) -> Self::V256 {
        double(self, || self.u8x16_splat(x))
    }

    #[inline(always)]
    fn u16x16_splat(self, x: u16) -> Self::V256 {
        double(self, || self.u16x8_splat(x))
    }

    #[inline(always)]
    fn u32x8_splat(self, x: u32) -> Self::V256 {
        double(self, || self.u32x4_splat(x))
    }

    #[inline(always)]
    fn u64x4_splat(self, x: u64) -> Self::V256 {
        double(self, || self.u64x2_splat(x))
    }

    #[inline(always)]
    fn i8x32_splat(self, x: i8) -> Self::V256 {
        double(self, || self.i8x16_splat(x))
    }

    #[inline(always)]
    fn i16x16_splat(self, x: i16) -> Self::V256 {
        double(self, || self.i16x8_splat(x))
    }

    #[inline(always)]
    fn i32x8_splat(self, x: i32) -> Self::V256 {
        double(self, || self.i32x4_splat(x))
    }

    #[inline(always)]
    fn i64x4_splat(self, x: i64) -> Self::V256 {
        double(self, || self.i64x2_splat(x))
    }

    #[inline(always)]
    fn u8x32_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u8x16_add)
    }

    #[inline(always)]
    fn u16x16_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u16x8_add)
    }

    #[inline(always)]
    fn u32x8_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u32x4_add)
    }

    #[inline(always)]
    fn u64x4_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u64x2_add)
    }

    #[inline(always)]
    fn u8x32_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u8x16_sub)
    }

    #[inline(always)]
    fn u16x16_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u16x8_sub)
    }

    #[inline(always)]
    fn u32x8_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u32x4_sub)
    }

    #[inline(always)]
    fn u64x4_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u64x2_sub)
    }

    #[inline(always)]
    fn u8x32_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u8x16_sub_sat)
    }

    #[inline(always)]
    fn u16x16_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u16x8_sub_sat)
    }

    #[inline(always)]
    fn i8x32_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i8x16_sub_sat)
    }

    #[inline(always)]
    fn i16x16_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i16x8_sub_sat)
    }

    #[inline(always)]
    fn u16x16_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        vmap(self, a, Self::u16x8_shl::<IMM8>)
    }

    #[inline(always)]
    fn u32x8_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        vmap(self, a, Self::u32x4_shl::<IMM8>)
    }

    #[inline(always)]
    fn u16x16_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        vmap(self, a, Self::u16x8_shr::<IMM8>)
    }

    #[inline(always)]
    fn u32x8_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        vmap(self, a, Self::u32x4_shr::<IMM8>)
    }

    #[inline(always)]
    fn u8x32_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u8x16_eq)
    }

    #[inline(always)]
    fn u16x16_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u16x8_eq)
    }

    #[inline(always)]
    fn u32x8_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u32x4_eq)
    }

    #[inline(always)]
    fn u8x32_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u8x16_lt)
    }

    #[inline(always)]
    fn u16x16_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u16x8_lt)
    }

    #[inline(always)]
    fn u32x8_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u32x4_lt)
    }

    #[inline(always)]
    fn i8x32_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i8x16_lt)
    }

    #[inline(always)]
    fn i16x16_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i16x8_lt)
    }

    #[inline(always)]
    fn i32x8_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i32x4_lt)
    }

    #[inline(always)]
    fn u8x32_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u8x16_max)
    }

    #[inline(always)]
    fn u16x16_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u16x8_max)
    }

    #[inline(always)]
    fn u32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u32x4_max)
    }

    #[inline(always)]
    fn i8x32_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i8x16_max)
    }

    #[inline(always)]
    fn i16x16_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i16x8_max)
    }

    #[inline(always)]
    fn i32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i32x4_max)
    }

    #[inline(always)]
    fn u8x32_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u8x16_min)
    }

    #[inline(always)]
    fn u16x16_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u16x8_min)
    }

    #[inline(always)]
    fn u32x8_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u32x4_min)
    }

    #[inline(always)]
    fn i8x32_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i8x16_min)
    }

    #[inline(always)]
    fn i16x16_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i16x8_min)
    }

    #[inline(always)]
    fn i32x8_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::i32x4_min)
    }

    #[inline(always)]
    fn u16x16_bswap(self, a: Self::V256) -> Self::V256 {
        vmap(self, a, Self::u16x8_bswap)
    }

    #[inline(always)]
    fn u32x8_bswap(self, a: Self::V256) -> Self::V256 {
        vmap(self, a, Self::u32x4_bswap)
    }

    #[inline(always)]
    fn u64x4_bswap(self, a: Self::V256) -> Self::V256 {
        vmap(self, a, Self::u64x2_bswap)
    }

    #[inline(always)]
    fn u8x16x2_swizzle(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        vmerge(self, a, b, Self::u8x16_swizzle)
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! inherit_simd256 {
    ($self: ty, $super: ty, $upcast: ident) => {
        unsafe impl SIMD256 for $self {
            type V256 = <$super as SIMD256>::V256;

            #[inline(always)]
            fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256 {
                <$super as SIMD256>::v256_from_v128x2(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn v256_to_v128x2(self, a: Self::V256) -> (Self::V128, Self::V128) {
                <$super as SIMD256>::v256_to_v128x2(self.$upcast(), a)
            }

            #[inline(always)]
            fn v256_to_bytes(self, a: Self::V256) -> [u8; 32] {
                <$super as SIMD256>::v256_to_bytes(self.$upcast(), a)
            }

            #[inline(always)]
            unsafe fn v256_load(self, addr: *const u8) -> Self::V256 {
                <$super as SIMD256>::v256_load(self.$upcast(), addr)
            }

            #[inline(always)]
            unsafe fn v256_load_unaligned(self, addr: *const u8) -> Self::V256 {
                <$super as SIMD256>::v256_load_unaligned(self.$upcast(), addr)
            }

            #[inline(always)]
            unsafe fn v256_store(self, addr: *mut u8, a: Self::V256) {
                <$super as SIMD256>::v256_store(self.$upcast(), addr, a)
            }

            #[inline(always)]
            unsafe fn v256_store_unaligned(self, addr: *mut u8, a: Self::V256) {
                <$super as SIMD256>::v256_store_unaligned(self.$upcast(), addr, a)
            }

            #[inline(always)]
            fn u8x32_splat(self, x: u8) -> Self::V256 {
                <$super as SIMD256>::u8x32_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u16x16_splat(self, x: u16) -> Self::V256 {
                <$super as SIMD256>::u16x16_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u32x8_splat(self, x: u32) -> Self::V256 {
                <$super as SIMD256>::u32x8_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u64x4_splat(self, x: u64) -> Self::V256 {
                <$super as SIMD256>::u64x4_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i8x32_splat(self, x: i8) -> Self::V256 {
                <$super as SIMD256>::i8x32_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i16x16_splat(self, x: i16) -> Self::V256 {
                <$super as SIMD256>::i16x16_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i32x8_splat(self, x: i32) -> Self::V256 {
                <$super as SIMD256>::i32x8_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn i64x4_splat(self, x: i64) -> Self::V256 {
                <$super as SIMD256>::i64x4_splat(self.$upcast(), x)
            }

            #[inline(always)]
            fn u8x32_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u8x32_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x16_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x8_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u32x8_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u64x4_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u64x4_add(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x32_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u8x32_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x16_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x8_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u32x8_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x32_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u8x32_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x16_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x32_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i8x32_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x16_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i16x16_sub_sat(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u64x4_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u64x4_sub(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x16_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_shl::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u32x8_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u32x8_shl::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u16x16_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_shr::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u32x8_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u32x8_shr::<IMM8>(self.$upcast(), a)
            }

            #[inline(always)]
            fn u8x32_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u8x32_eq(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x16_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_eq(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x8_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u32x8_eq(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x32_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u8x32_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x16_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x8_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u32x8_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x32_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i8x32_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x16_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i16x16_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x8_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i32x8_lt(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x32_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u8x32_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x16_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u32x8_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x32_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i8x32_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x16_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i16x16_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i32x8_max(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x32_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u8x32_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x16_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u32x8_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u32x8_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i8x32_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i8x32_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i16x16_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i16x16_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn i32x8_min(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::i32x8_min(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u8x16x2_swizzle(self, a: Self::V256, b: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u8x16x2_swizzle(self.$upcast(), a, b)
            }

            #[inline(always)]
            fn u16x16_bswap(self, a: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u16x16_bswap(self.$upcast(), a)
            }

            #[inline(always)]
            fn u32x8_bswap(self, a: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u32x8_bswap(self.$upcast(), a)
            }

            #[inline(always)]
            fn u64x4_bswap(self, a: Self::V256) -> Self::V256 {
                <$super as SIMD256>::u64x4_bswap(self.$upcast(), a)
            }
        }
    };
}
