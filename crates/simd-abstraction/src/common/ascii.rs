use crate::isa::{SimdLoad, SIMD256};
use crate::scalar::{align32, Bytes32};
use crate::tools::unroll;

use self::spec::SIMDExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AsciiCase {
    Lower = 0,
    Upper = 1,
}

#[inline]
#[must_use]
pub fn is_ascii_ct_fallback(data: &[u8]) -> bool {
    let mut ans = 0;
    unroll(data, 8, |&x| ans |= x);
    ans < 0x80
}

#[inline]
pub fn is_ascii_ct_simd<S: SIMDExt>(s: S, data: &[u8]) -> bool {
    let (prefix, middle, suffix) = align32(data);

    let mut ans = is_ascii_ct_fallback(prefix);

    let mut mask = s.v256_create_zero();
    unroll(middle, 8, |chunk| mask = s.v256_or(mask, s.load(chunk)));
    ans &= s.is_ascii_u8x32(mask);

    ans &= is_ascii_ct_fallback(suffix);
    ans
}

#[inline(always)]
fn lookup_ascii_whitespace(c: u8) -> u8 {
    const TABLE: &[u8; 256] = &{
        let mut ans = [0; 256];
        let mut i: u8 = 0;
        loop {
            ans[i as usize] = if i.is_ascii_whitespace() { 0xff } else { 0 };
            if i == 255 {
                break;
            }
            i += 1;
        }
        ans
    };
    unsafe { *TABLE.get_unchecked(c as usize) }
}

#[inline]
#[must_use]
pub fn find_non_ascii_whitespace_fallback(data: &[u8]) -> usize {
    unsafe {
        let n = data.len();
        let mut src = data.as_ptr();

        const UNROLL: usize = 8;
        let end = src.add(n / UNROLL * UNROLL);
        while src < end {
            let mut flag = 0;
            for _ in 0..UNROLL {
                flag |= lookup_ascii_whitespace(src.read());
                src = src.add(1)
            }
            if flag != 0 {
                src = src.sub(UNROLL);
                break;
            }
        }

        let end = data.as_ptr().add(n);
        while src < end {
            if lookup_ascii_whitespace(src.read()) != 0 {
                break;
            }
            src = src.add(1);
        }

        src.offset_from(data.as_ptr()) as usize
    }
}

#[inline(always)]
fn check_non_ascii_whitespace_u8x32<S: SIMD256>(s: S, a: S::V256) -> bool {
    // ASCII whitespaces
    // TAB      0x09    00001001
    // LF       0x0a    00001010
    // FF       0x0c    00001100
    // CR       0x0d    00001101
    // SPACE    0x20    00010000
    //

    const LUT: &Bytes32 = &Bytes32::double([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
        0xff, 0x00, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, //
    ]);

    let lut: _ = s.load(LUT);

    let m1: _ = s.u8x16x2_swizzle(lut, a);
    let m2: _ = s.v256_and(a, s.u8x32_splat(0xf0));
    let m3: _ = s.u8x32_eq(s.v256_or(m1, m2), s.v256_create_zero());
    let m4: _ = s.u8x32_eq(a, s.u8x32_splat(0x20));
    let m5: _ = s.v256_or(m3, m4);

    !s.v256_all_zero(m5)
}

#[inline]
pub fn find_non_ascii_whitespace_simd<S: SIMD256>(s: S, data: &[u8]) -> usize {
    let (prefix, middle, suffix) = align32(data);

    let mut pos: usize = 0;

    {
        let offset = find_non_ascii_whitespace_fallback(prefix);
        pos = pos.wrapping_add(offset);
        if offset != prefix.len() {
            return pos;
        }
    }

    for chunk in middle {
        if check_non_ascii_whitespace_u8x32(s, s.load(chunk)) {
            let offset = find_non_ascii_whitespace_fallback(&chunk.0);
            pos += offset;
            return pos;
        }
        pos += 32;
    }

    {
        let offset = find_non_ascii_whitespace_fallback(suffix);
        pos = pos.wrapping_add(offset);
    }

    pos
}

#[inline]
pub unsafe fn remove_ascii_whitespace_fallback(data: *mut u8, len: usize) -> usize {
    let mut src: *const u8 = data;
    let mut dst: *mut u8 = data;
    let end: *const u8 = data.add(len);

    while src < end {
        let byte = src.read();
        if lookup_ascii_whitespace(byte) == 0 {
            dst.write(byte);
            dst = dst.add(1);
        }
        src = src.add(1);
    }

    dst.offset_from(data) as usize
}

pub mod multiversion {
    use super::*;

    crate::simd_dispatch! (
        name        = is_ascii_ct,
        signature   = fn(data: &[u8]) -> bool,
        fallback    = {is_ascii_ct_fallback},
        simd        = {is_ascii_ct_simd},
        safety      = {},
    );

    crate::simd_dispatch!(
        name        = find_non_ascii_whitespace,
        signature   = fn(data: &[u8]) -> usize,
        fallback    = {find_non_ascii_whitespace_fallback},
        simd        = {find_non_ascii_whitespace_simd},
        safety      = {},
    );
}

mod spec {
    use crate::isa::SIMD256;

    pub unsafe trait SIMDExt: SIMD256 {
        fn is_ascii_u8x32(self, a: Self::V256) -> bool;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    mod x86 {
        use super::*;

        #[cfg(target_arch = "x86")]
        use core::arch::x86::*;

        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::*;

        use crate::arch::x86::*;
        use crate::isa::SIMD128;

        unsafe impl SIMDExt for AVX2 {
            #[inline(always)]
            fn is_ascii_u8x32(self, a: Self::V256) -> bool {
                unsafe { _mm256_movemask_epi8(a) == 0 }
            }
        }

        unsafe impl SIMDExt for SSE41 {
            #[inline(always)]
            fn is_ascii_u8x32(self, a: Self::V256) -> bool {
                let x = self.v128_or(a.0, a.1);
                let m = unsafe { _mm_movemask_epi8(x) };
                m == 0
            }
        }
    }

    #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
    mod arm {
        use super::*;

        use crate::arch::arm::*;
        use crate::isa::SIMD128;

        #[cfg(target_arch = "aarch64")]
        use core::arch::aarch64::*;

        unsafe impl SIMDExt for NEON {
            #[inline(always)]
            fn is_ascii_u8x32(self, a: Self::V256) -> bool {
                let x = self.v128_or(a.0, a.1);

                #[cfg(target_arch = "arm")]
                {
                    self.v128_all_zero(self.i32x4_lt(x, self.v128_create_zero()))
                }
                #[cfg(target_arch = "aarch64")]
                unsafe {
                    vmaxvq_u8(x) < 0x80
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    mod wasm {
        use super::*;

        use crate::arch::wasm::*;
        use crate::isa::SIMD128 as _;

        use core::arch::wasm32::*;

        unsafe impl SIMDExt for SIMD128 {
            #[inline(always)]
            fn is_ascii_u8x32(self, a: Self::V256) -> bool {
                let x = self.v128_or(a.0, a.1);
                let m = i8x16_bitmask(x);
                m == 0
            }
        }
    }
}
