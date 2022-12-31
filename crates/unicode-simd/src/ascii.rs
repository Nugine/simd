use vsimd::mask::u8x32_highbit_any;
use vsimd::SIMD256;

use core::ops::Not;

#[inline(always)]
pub unsafe fn is_ascii_fallback(mut src: *const u8, len: usize) -> bool {
    let mut ans = 0;
    let end = src.add(len);
    while src < end {
        ans |= src.read();
        src = src.add(1);
    }
    ans < 0x80
}

#[inline(always)]
pub unsafe fn is_ascii_simd<S: SIMD256>(s: S, src: *const u8, len: usize) -> bool {
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    {
        use vsimd::isa::SSE2;
        use vsimd::matches_isa;

        if matches_isa!(S, SSE2) {
            return is_ascii_sse2(src, len);
        }
    }

    is_ascii_simd_v256(s, src, len)
}

#[inline(always)]
pub unsafe fn is_ascii_simd_v256<S: SIMD256>(s: S, mut src: *const u8, mut len: usize) -> bool {
    let end = src.add(len / 32 * 32);
    let mut y = s.v256_create_zero();
    while src < end {
        let x = s.v256_load_unaligned(src);
        y = s.v256_or(y, x);
        src = src.add(32);
    }
    len %= 32;

    let mut ans = u8x32_highbit_any(s, y).not();
    ans &= is_ascii_fallback(src, len);
    ans
}

#[allow(clippy::too_many_lines)]
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline]
#[must_use]
pub unsafe fn is_ascii_sse2(src: *const u8, len: usize) -> bool {
    use core::arch::x86_64::*;

    macro_rules! ensure {
        ($cond:expr) => {
            if !$cond {
                return false;
            }
        };
    }

    #[inline(always)]
    unsafe fn loadu<T>(p: *const u8) -> T {
        p.cast::<T>().read_unaligned()
    }

    #[inline(always)]
    fn check4(x: u32) -> bool {
        (x & 0x8080_8080) == 0
    }

    #[inline(always)]
    fn check8(x: u64) -> bool {
        (x & 0x8080_8080_8080_8080) == 0
    }

    #[inline(always)]
    unsafe fn check16(x: __m128i) -> bool {
        if cfg!(miri) {
            let x = core::mem::transmute(x);
            vsimd::simulation::u8x16_bitmask(x) == 0
        } else {
            _mm_movemask_epi8(x) as u32 as u16 == 0
        }
    }

    #[inline(always)]
    unsafe fn or(a: __m128i, b: __m128i) -> __m128i {
        _mm_or_si128(a, b)
    }

    /// len in 0..=8
    #[inline(always)]
    unsafe fn check_tiny(mut src: *const u8, mut len: usize) -> bool {
        if len == 8 {
            return check8(loadu(src));
        }
        if len >= 4 {
            ensure!(check4(loadu(src)));
            src = src.add(4);
            len -= 4;
        }
        {
            let mut acc: u8 = 0;
            let end = src.add(len);
            for _ in 0..3 {
                if src < end {
                    acc |= src.read();
                    src = src.add(1);
                }
            }
            acc < 0x80
        }
    }

    /// len in 9..=16
    #[inline(always)]
    unsafe fn check_short(src: *const u8, len: usize) -> bool {
        let x1: u64 = loadu(src);
        let x2: u64 = loadu(src.add(len - 8));
        check8(x1 | x2)
    }

    /// len in 17..64
    #[inline(always)]
    unsafe fn check_medium(src: *const u8, len: usize) -> bool {
        let mut x: __m128i = loadu(src);
        if len >= 32 {
            x = or(x, loadu(src.add(16)));
        }
        if len >= 48 {
            x = or(x, loadu(src.add(32)));
        }
        x = or(x, loadu(src.add(len - 16)));
        check16(x)
    }

    /// len >= 64
    #[inline(always)]
    unsafe fn check_long(mut src: *const u8, mut len: usize) -> bool {
        let end = src.add(len / 64 * 64);
        while src < end {
            let x: [__m128i; 4] = loadu(src);
            ensure!(check16(or(or(x[0], x[1]), or(x[2], x[3]))));
            src = src.add(64);
        }
        len %= 64;
        if len == 0 {
            return true;
        }
        if len <= 8 {
            check_tiny(src, len)
        } else if len <= 16 {
            check_short(src, len)
        } else {
            check_medium(src, len)
        }
    }

    {
        if len <= 8 {
            check_tiny(src, len)
        } else if len <= 16 {
            check_short(src, len)
        } else if len < 64 {
            check_medium(src, len)
        } else {
            check_long(src, len)
        }
    }
}
