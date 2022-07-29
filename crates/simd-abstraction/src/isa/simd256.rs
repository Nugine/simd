use super::SIMD128;

#[inline(always)]
fn split_merge<S: SIMD256>(
    s: S,
    a: S::V256,
    b: S::V256,
    f: impl FnOnce((S::V128, S::V128), (S::V128, S::V128)) -> (S::V128, S::V128),
) -> S::V256 {
    let a = s.v256_to_v128x2(a);
    let b = s.v256_to_v128x2(b);
    let (c0, c1) = f(a, b);
    s.v256_from_v128x2(c0, c1)
}

pub unsafe trait SIMD256: SIMD128 {
    type V256: Copy;

    fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256;
    fn v256_to_v128x2(self, a: Self::V256) -> (Self::V128, Self::V128);
    fn v256_to_bytes(self, a: Self::V256) -> [u8; 32];

    fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256;

    fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128;

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
    unsafe fn v256_store_unaligned(self, addr: *mut u8, a: Self::V256) {
        let (a0, a1) = self.v256_to_v128x2(a);
        self.v128_store_unaligned(addr, a0);
        self.v128_store_unaligned(addr.add(16), a1);
    }

    #[inline(always)]
    fn v256_or(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.v128_or(a.0, b.0), self.v128_or(a.1, b.1))
        })
    }

    #[inline(always)]
    fn v256_and(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.v128_and(a.0, b.0), self.v128_and(a.1, b.1))
        })
    }

    #[inline(always)]
    fn v256_andnot(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.v128_andnot(a.0, b.0), self.v128_andnot(a.1, b.1))
        })
    }

    #[inline(always)]
    fn v256_xor(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.v128_xor(a.0, b.0), self.v128_xor(a.1, b.1))
        })
    }

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
    fn u8x32_splat(self, x: u8) -> Self::V256 {
        self.v256_from_v128x2(self.u8x16_splat(x), self.u8x16_splat(x))
    }

    #[inline(always)]
    fn u8x32_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.u8x16_add(a.0, b.0), self.u8x16_add(a.1, b.1))
        })
    }

    #[inline(always)]
    fn u8x32_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.u8x16_sub(a.0, b.0), self.u8x16_sub(a.1, b.1))
        })
    }

    #[inline(always)]
    fn u8x32_any_zero(self, a: Self::V256) -> bool {
        let a = self.v256_to_v128x2(a);
        self.u8x16_any_zero(self.u8x16_min(a.0, a.1))
    }

    #[inline(always)]
    fn u8x16x2_swizzle(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.u8x16_swizzle(a.0, b.0), self.u8x16_swizzle(a.1, b.1))
        })
    }

    #[inline(always)]
    fn i8x32_splat(self, x: i8) -> Self::V256 {
        self.v256_from_v128x2(self.i8x16_splat(x), self.i8x16_splat(x))
    }

    #[inline(always)]
    fn i8x32_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.i8x16_lt(a.0, b.0), self.i8x16_lt(a.1, b.1))
        })
    }

    #[inline(always)]
    fn i8x32_eq(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.i8x16_eq(a.0, b.1), self.i8x16_eq(a.1, b.1))
        })
    }

    #[inline(always)]
    fn u16x16_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        let a = self.v256_to_v128x2(a);
        self.v256_from_v128x2(self.u16x8_shl::<IMM8>(a.0), self.u16x8_shl::<IMM8>(a.1))
    }

    #[inline(always)]
    fn u16x16_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        let a = self.v256_to_v128x2(a);
        self.v256_from_v128x2(self.u16x8_shr::<IMM8>(a.0), self.u16x8_shr::<IMM8>(a.1))
    }

    #[inline(always)]
    fn u16x16_splat(self, x: u16) -> Self::V256 {
        self.v256_from_v128x2(self.u16x8_splat(x), self.u16x8_splat(x))
    }

    #[inline(always)]
    fn u32x8_splat(self, x: u32) -> Self::V256 {
        self.v256_from_v128x2(self.u32x4_splat(x), self.u32x4_splat(x))
    }

    #[inline(always)]
    fn u8x32_sub_sat(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.u8x16_sub_sat(a.0, b.0), self.u8x16_sub_sat(a.1, b.1))
        })
    }

    #[inline(always)]
    fn u32x8_shl<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        let a = self.v256_to_v128x2(a);
        self.v256_from_v128x2(self.u32x4_shl::<IMM8>(a.0), self.u32x4_shl::<IMM8>(a.1))
    }

    #[inline(always)]
    fn u32x8_shr<const IMM8: i32>(self, a: Self::V256) -> Self::V256 {
        let a = self.v256_to_v128x2(a);
        self.v256_from_v128x2(self.u32x4_shr::<IMM8>(a.0), self.u32x4_shr::<IMM8>(a.1))
    }

    #[inline(always)]
    fn u32x8_add(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.u32x4_add(a.0, b.0), self.u32x4_add(a.1, b.1))
        })
    }

    #[inline(always)]
    fn u32x8_sub(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.u32x4_sub(a.0, b.0), self.u32x4_sub(a.1, b.1))
        })
    }

    #[inline(always)]
    fn u32x8_max(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.u32x4_max(a.0, b.0), self.u32x4_max(a.1, b.1))
        })
    }

    #[inline(always)]
    fn u32x8_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.u32x4_lt(a.0, b.0), self.u32x4_lt(a.1, b.1))
        })
    }

    #[inline(always)]
    fn u32x8_bswap(self, a: Self::V256) -> Self::V256 {
        let a = self.v256_to_v128x2(a);
        self.v256_from_v128x2(self.u32x4_bswap(a.0), self.u32x4_bswap(a.1))
    }

    #[inline(always)]
    fn i32x8_lt(self, a: Self::V256, b: Self::V256) -> Self::V256 {
        split_merge(self, a, b, |a, b| {
            (self.i32x4_lt(a.0, b.0), self.i32x4_lt(a.1, b.1))
        })
    }
}
