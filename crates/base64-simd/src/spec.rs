use simd_abstraction::isa::SIMD256;

#[allow(missing_docs, clippy::missing_safety_doc)]
pub unsafe trait SIMDExt: SIMD256 {
    fn base64_split_bits(self, a: Self::V256) -> Self::V256 {
        // a : {bbbbcccc|aaaaaabb|ccdddddd|bbbbcccc} x8 (1021)

        let m1 = self.u32x8_splat(u32::from_le_bytes([0x00, 0xfc, 0x00, 0x00]));
        let a1 = self.u16x16_shr::<10>(self.v256_and(a, m1));
        // a1: {00aaaaaa|000000000|00000000|00000000} x8

        let m2 = self.u32x8_splat(u32::from_le_bytes([0xf0, 0x03, 0x00, 0x00]));
        let a2 = self.u16x16_shl::<4>(self.v256_and(a, m2));
        // a2: {00000000|00bbbbbb|00000000|00000000} x8

        let m3 = self.u32x8_splat(u32::from_le_bytes([0x00, 0x00, 0xc0, 0x0f]));
        let a3 = self.u16x16_shr::<6>(self.v256_and(a, m3));
        // a3: {00000000|00000000|00cccccc|00000000} x8

        let m4 = self.u32x8_splat(u32::from_le_bytes([0x00, 0x00, 0x3f, 0x00]));
        let a4 = self.u16x16_shl::<8>(self.v256_and(a, m4));
        // a4: {00000000|00000000|00000000|00dddddd} x8

        self.v256_or(self.v256_or(a1, a2), self.v256_or(a3, a4))
        // {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8
    }

    fn base64_merge_bits(self, a: Self::V256) -> Self::V256 {
        // a : {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8

        let m1 = self.u32x8_splat(u32::from_le_bytes([0x3f, 0x00, 0x3f, 0x00]));
        let a1 = self.v256_and(a, m1);
        // a1: {00aaaaaa|00000000|00cccccc|00000000} x8

        let m2 = self.u32x8_splat(u32::from_le_bytes([0x00, 0x3f, 0x00, 0x3f]));
        let a2 = self.v256_and(a, m2);
        // a2: {00000000|00bbbbbb|00000000|00dddddd} x8

        let a3 = self.v256_or(self.u32x8_shl::<18>(a1), self.u32x8_shr::<10>(a1));
        // a3: {cc000000|0000cccc|aaaaaa00|00000000} x8

        let a4 = self.v256_or(self.u32x8_shl::<4>(a2), self.u32x8_shr::<24>(a2));
        // a4: {00dddddd|bbbb0000|000000bb|dddd0000}

        let mask = self.u32x8_splat(u32::from_le_bytes([0xff, 0xff, 0xff, 0x00]));
        self.v256_and(self.v256_or(a3, a4), mask)
        // {ccdddddd|bbbbcccc|aaaaaabb|00000000} x8
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    use super::SIMDExt;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;

    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    use simd_abstraction::arch::x86::*;
    use simd_abstraction::isa::{SIMD128, SIMD256};

    const SPLIT_M1: u32 = u32::from_le_bytes([0x00, 0xfc, 0xc0, 0x0f]);
    const SPLIT_M2: u32 = u32::from_le_bytes([0xf0, 0x03, 0x3f, 0x00]);
    const SPLIT_M3: u32 = u32::from_le_bytes([0x40, 0x00, 0x00, 0x04]);
    const SPLIT_M4: u32 = u32::from_le_bytes([0x10, 0x00, 0x00, 0x01]);

    const MERGE_M1: u16 = u16::from_le_bytes([0x40, 0x01]);
    const MERGE_M2: u32 = u32::from_le_bytes([0x00, 0x10, 0x01, 0x00]);

    unsafe impl SIMDExt for AVX2 {
        #[inline(always)]
        fn base64_split_bits(self, a: Self::V256) -> Self::V256 {
            // a : {bbbbcccc|aaaaaabb|ccdddddd|bbbbcccc} x8 (1021)

            let a1 = self.v256_and(a, self.u32x8_splat(SPLIT_M1));
            // a1: {00000000|aaaaaa00|cc000000|0000cccc} x8

            let a2 = self.v256_and(a, self.u32x8_splat(SPLIT_M2));
            // a2: {bbbb0000|000000bb|00dddddd|00000000} x8

            let a3 = unsafe { _mm256_mulhi_epu16(a1, self.u32x8_splat(SPLIT_M3)) };
            // a3: {00aaaaaa|00000000|00cccccc|00000000} x8

            let a4 = unsafe { _mm256_mullo_epi16(a2, self.u32x8_splat(SPLIT_M4)) };
            // a4: {00000000|00bbbbbb|00000000|00dddddd} x8

            self.v256_or(a3, a4)
            // {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8
        }

        #[inline(always)]
        fn base64_merge_bits(self, a: Self::V256) -> Self::V256 {
            // a : {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8

            let a1 = unsafe { _mm256_maddubs_epi16(a, self.u16x16_splat(MERGE_M1)) };
            // a1: {aabbbbbb|0000aaaa|ccdddddd|0000cccc} x8

            unsafe { _mm256_madd_epi16(a1, self.u32x8_splat(MERGE_M2)) }
            // {00000000|aaaaaabb|bbbbcccc|ccdddddd} x8
        }
    }

    unsafe impl SIMDExt for SSE41 {
        #[inline(always)]
        fn base64_split_bits(self, a: Self::V256) -> Self::V256 {
            // a : {bbbbcccc|aaaaaabb|ccdddddd|bbbbcccc} x8 (1021)

            let a1 = self.v256_and(a, self.u32x8_splat(SPLIT_M1));
            // a1: {00000000|aaaaaa00|cc000000|0000cccc} x8

            let a2 = self.v256_and(a, self.u32x8_splat(SPLIT_M2));
            // a2: {bbbb0000|000000bb|00dddddd|00000000} x8

            let a3 = unsafe {
                let m3 = self.u32x4_splat(SPLIT_M3);
                (_mm_mulhi_epu16(a1.0, m3), _mm_mulhi_epu16(a1.1, m3))
            };
            // a3: {00aaaaaa|00000000|00cccccc|00000000} x8

            let a4 = unsafe {
                let m4 = self.u32x4_splat(SPLIT_M4);
                (_mm_mullo_epi16(a2.0, m4), _mm_mullo_epi16(a2.1, m4))
            };
            // a4: {00000000|00bbbbbb|00000000|00dddddd} x8

            self.v256_or(a3, a4)
            // {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8
        }

        #[inline(always)]
        fn base64_merge_bits(self, a: Self::V256) -> Self::V256 {
            // a : {00aaaaaa|00bbbbbb|00cccccc|00dddddd} x8

            let a1 = unsafe {
                let m1 = self.u16x8_splat(MERGE_M1);
                (_mm_maddubs_epi16(a.0, m1), _mm_maddubs_epi16(a.1, m1))
            };
            // a1: {aabbbbbb|0000aaaa|ccdddddd|0000cccc} x8

            unsafe {
                let m2 = self.u32x4_splat(MERGE_M2);
                (_mm_madd_epi16(a1.0, m2), _mm_madd_epi16(a1.1, m2))
            }
            // {ccdddddd|bbbbcccc|aaaaaabb|00000000} x8
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::SIMDExt;

    use simd_abstraction::arch::wasm::*;

    unsafe impl SIMDExt for SIMD128 {}
}

#[cfg(all(
    feature = "unstable",
    any(target_arch = "arm", target_arch = "aarch64")
))]
mod arm {
    use super::SIMDExt;

    #[cfg(target_arch = "arm")]
    use simd_abstraction::arch::arm::*;

    #[cfg(target_arch = "aarch64")]
    use simd_abstraction::arch::aarch64::*;

    unsafe impl SIMDExt for NEON {} // TODO: better ways?
}
