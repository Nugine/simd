use super::*;

impl SSE42 {
    #[inline(always)]
    fn sse41(self) -> SSE41 {
        unsafe { SSE41::new() }
    }
}

unsafe impl SIMD128 for SSE42 {
    type V128 = <SSE41 as SIMD128>::V128;

    #[inline(always)]
    unsafe fn v128_load(self, addr: *const u8) -> Self::V128 {
        self.sse41().v128_load(addr)
    }

    #[inline(always)]
    unsafe fn v128_load_unaligned(self, addr: *const u8) -> Self::V128 {
        self.sse41().v128_load_unaligned(addr)
    }

    #[inline(always)]
    unsafe fn v128_store_unaligned(self, addr: *mut u8, a: Self::V128) {
        self.sse41().v128_store_unaligned(addr, a)
    }

    #[inline(always)]
    fn v128_or(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().v128_or(a, b)
    }

    #[inline(always)]
    fn v128_and(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().v128_and(a, b)
    }

    #[inline(always)]
    fn v128_to_bytes(self, a: Self::V128) -> [u8; 16] {
        self.sse41().v128_to_bytes(a)
    }

    #[inline(always)]
    fn v128_create_zero(self) -> Self::V128 {
        self.sse41().v128_create_zero()
    }

    #[inline(always)]
    fn v128_all_zero(self, a: Self::V128) -> bool {
        self.sse41().v128_all_zero(a)
    }

    #[inline(always)]
    fn v128_andnot(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().v128_andnot(a, b)
    }

    #[inline(always)]
    fn v128_xor(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().v128_xor(a, b)
    }

    #[inline(always)]
    fn u8x16_splat(self, x: u8) -> Self::V128 {
        self.sse41().u8x16_splat(x)
    }

    #[inline(always)]
    fn u8x16_swizzle(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_swizzle(a, b)
    }

    #[inline(always)]
    fn u8x16_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_add(a, b)
    }

    #[inline(always)]
    fn u8x16_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_sub(a, b)
    }

    #[inline(always)]
    fn u8x16_sub_sat(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_sub_sat(a, b)
    }

    #[inline(always)]
    fn u8x16_any_zero(self, a: Self::V128) -> bool {
        self.sse41().u8x16_any_zero(a)
    }

    #[inline(always)]
    fn u8x16_min(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u8x16_min(a, b)
    }

    #[inline(always)]
    fn i8x16_splat(self, x: i8) -> Self::V128 {
        self.sse41().i8x16_splat(x)
    }

    #[inline(always)]
    fn i8x16_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().i8x16_lt(a, b)
    }

    #[inline(always)]
    fn i8x16_eq(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().i8x16_eq(a, b)
    }

    #[inline(always)]
    fn u16x8_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        self.sse41().u16x8_shl::<IMM8>(a)
    }

    #[inline(always)]
    fn u16x8_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        self.sse41().u16x8_shr::<IMM8>(a)
    }

    #[inline(always)]
    fn u16x8_splat(self, x: u16) -> Self::V128 {
        self.sse41().u16x8_splat(x)
    }

    #[inline(always)]
    fn u16x8_bswap(self, a: Self::V128) -> Self::V128 {
        self.sse41().u16x8_bswap(a)
    }

    #[inline(always)]
    fn u32x4_splat(self, x: u32) -> Self::V128 {
        self.sse41().u32x4_splat(x)
    }

    #[inline(always)]
    fn u32x4_shl<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        self.sse41().u32x4_shl::<IMM8>(a)
    }

    #[inline(always)]
    fn u32x4_shr<const IMM8: i32>(self, a: Self::V128) -> Self::V128 {
        self.sse41().u32x4_shr::<IMM8>(a)
    }

    #[inline(always)]
    fn u32x4_add(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u32x4_add(a, b)
    }

    #[inline(always)]
    fn u32x4_sub(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u32x4_sub(a, b)
    }

    #[inline(always)]
    fn u32x4_max(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u32x4_max(a, b)
    }

    #[inline(always)]
    fn u32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().u32x4_lt(a, b)
    }

    #[inline(always)]
    fn u32x4_bswap(self, a: Self::V128) -> Self::V128 {
        self.sse41().u32x4_bswap(a)
    }

    #[inline(always)]
    fn i32x4_lt(self, a: Self::V128, b: Self::V128) -> Self::V128 {
        self.sse41().i32x4_lt(a, b)
    }

    #[inline(always)]
    fn u64x2_bswap(self, a: Self::V128) -> Self::V128 {
        self.sse41().u64x2_bswap(a)
    }
}

unsafe impl SIMD256 for SSE42 {
    type V256 = <SSE41 as SIMD256>::V256;

    #[inline(always)]
    fn v256_to_bytes(self, a: Self::V256) -> [u8; 32] {
        self.sse41().v256_to_bytes(a)
    }

    #[inline(always)]
    fn u16x16_from_u8x16(self, a: Self::V128) -> Self::V256 {
        self.sse41().u16x16_from_u8x16(a)
    }

    #[inline(always)]
    fn u64x4_unzip_low(self, a: Self::V256) -> Self::V128 {
        self.sse41().u64x4_unzip_low(a)
    }

    #[inline(always)]
    fn v256_from_v128x2(self, a: Self::V128, b: Self::V128) -> Self::V256 {
        self.sse41().v256_from_v128x2(a, b)
    }

    #[inline(always)]
    fn v256_to_v128x2(self, a: Self::V256) -> (Self::V128, Self::V128) {
        self.sse41().v256_to_v128x2(a)
    }
}
