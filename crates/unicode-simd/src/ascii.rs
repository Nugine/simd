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
        use vsimd::isa::{AVX2, SSE2};
        use vsimd::matches_isa;

        #[cfg(not(target_feature = "avx2"))]
        {
            if matches_isa!(S, AVX2) {
                return is_ascii_avx2(s, src, len);
            }
        }

        if matches_isa!(S, SSE2) {
            return is_ascii_sse2(src, len);
        }
    }

    is_ascii_simd_v256(s, src, len)
}

#[inline(always)]
pub unsafe fn is_ascii_simd_v256<S: SIMD256>(s: S, mut src: *const u8, mut len: usize) -> bool {
    // Reduce and branch every 128 bytes so an invalid byte is rejected after a bounded amount
    // of work instead of after scanning the whole input, while the four loads per iteration
    // keep enough independent work in flight.
    let block_end = src.add(len / 128 * 128);
    while src < block_end {
        let x0 = s.v256_load_unaligned(src);
        let x1 = s.v256_load_unaligned(src.add(32));
        let x2 = s.v256_load_unaligned(src.add(64));
        let x3 = s.v256_load_unaligned(src.add(96));
        let y = s.v256_or(s.v256_or(x0, x1), s.v256_or(x2, x3));
        if u8x32_highbit_any(s, y) {
            return false;
        }
        src = src.add(128);
    }
    len %= 128;

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

    use vsimd::vector::V128;

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
            let x = core::mem::transmute::<__m128i, V128>(x);
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
        #[cfg(target_feature = "avx2")]
        {
            let end = src.add(len / 64 * 64);
            while src < end {
                let x0 = _mm256_loadu_si256(src.cast());
                let x1 = _mm256_loadu_si256(src.add(32).cast());
                ensure!(_mm256_movemask_epi8(_mm256_or_si256(x0, x1)) == 0);
                src = src.add(64);
            }
        }

        #[cfg(not(target_feature = "avx2"))]
        {
            let end = src.add(len / 64 * 64);
            while src < end {
                let x: [__m128i; 4] = loadu(src);
                ensure!(check16(or(or(x[0], x[1]), or(x[2], x[3]))));
                src = src.add(64);
            }
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

/// AVX2 ASCII validator mirroring the size tiers and early-exit behavior of [`is_ascii_sse2`],
/// but using 256-bit blocks above 64 bytes and a 128-byte main loop.
#[allow(clippy::too_many_lines)]
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline]
#[must_use]
unsafe fn is_ascii_avx2<S: SIMD256>(s: S, src: *const u8, len: usize) -> bool {
    use vsimd::mask::u8x16_highbit_any;
    use vsimd::vector::{V128, V256};

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
    fn check16<S: SIMD256>(s: S, x: V128) -> bool {
        u8x16_highbit_any(s, x).not()
    }

    #[inline(always)]
    fn check32<S: SIMD256>(s: S, x: V256) -> bool {
        u8x32_highbit_any(s, x).not()
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

    /// len in 17..=64. 128-bit blocks measured faster than 256-bit ones in this band.
    #[inline(always)]
    unsafe fn check_medium<S: SIMD256>(s: S, src: *const u8, len: usize) -> bool {
        let mut x = s.v128_load_unaligned(src);
        if len >= 32 {
            x = s.v128_or(x, s.v128_load_unaligned(src.add(16)));
        }
        if len >= 48 {
            x = s.v128_or(x, s.v128_load_unaligned(src.add(32)));
        }
        x = s.v128_or(x, s.v128_load_unaligned(src.add(len - 16)));
        check16(s, x)
    }

    /// len in 65..128
    #[inline(always)]
    unsafe fn check_wide<S: SIMD256>(s: S, src: *const u8, len: usize) -> bool {
        let mut x = s.v256_load_unaligned(src);
        x = s.v256_or(x, s.v256_load_unaligned(src.add(32)));
        if len >= 96 {
            x = s.v256_or(x, s.v256_load_unaligned(src.add(64)));
        }
        x = s.v256_or(x, s.v256_load_unaligned(src.add(len - 32)));
        check32(s, x)
    }

    /// len >= 128
    #[inline(always)]
    unsafe fn check_long<S: SIMD256>(s: S, mut src: *const u8, mut len: usize) -> bool {
        let end = src.add(len / 64 * 64);
        while src < end {
            let x0 = s.v256_load_unaligned(src);
            let x1 = s.v256_load_unaligned(src.add(32));
            ensure!(check32(s, s.v256_or(x0, x1)));
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
        } else if len <= 64 {
            check_medium(s, src, len)
        } else {
            check_wide(s, src, len)
        }
    }

    {
        if len <= 8 {
            check_tiny(src, len)
        } else if len <= 16 {
            check_short(src, len)
        } else if len <= 64 {
            check_medium(s, src, len)
        } else if len < 128 {
            check_wide(s, src, len)
        } else {
            check_long(s, src, len)
        }
    }
}
