use crate::Error;

use vsimd::hex::unhex;
use vsimd::tools::{slice, slice_parts};
use vsimd::SIMD256;

#[inline(always)]
fn check_short(data: &[u8]) -> Result<(), Error> {
    // FIXME:
    // The later version triggers incorrect auto-vectorization when avx2 is enabled.
    // https://github.com/Nugine/simd/issues/14
    // https://github.com/rust-lang/rust/issues/102709
    //

    if cfg!(target_feature = "avx2") {
        for &x in data {
            ensure!(unhex(x) != 0xff);
        }
    } else {
        let mut flag = 0;
        for &x in data {
            flag |= unhex(x);
        }
        ensure!(flag != 0xff);
    }

    Ok(())
}

#[inline(always)]
pub fn check_fallback(data: &[u8]) -> Result<(), Error> {
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
    unsafe {
        if cfg!(target_feature = "sse2") {
            return self::sse2::check(data);
        }
    }
    check_short(data)
}

#[inline(always)]
pub fn check_simd<S: SIMD256>(s: S, data: &[u8]) -> Result<(), Error> {
    unsafe {
        let (mut src, mut len) = slice_parts(data);

        if len == 16 {
            let x = s.v128_load_unaligned(src);
            ensure!(vsimd::hex::check_xn(s, x));
            return Ok(());
        }

        if len == 32 {
            let x = s.v256_load_unaligned(src);
            ensure!(vsimd::hex::check_xn(s, x));
            return Ok(());
        }

        let end = src.add(len / 32 * 32);
        while src < end {
            let x = s.v256_load_unaligned(src);
            ensure!(vsimd::hex::check_xn(s, x));
            src = src.add(32);
        }
        len %= 32;

        if len == 0 {
            return Ok(());
        }

        if len >= 16 {
            let x = s.v128_load_unaligned(src);
            ensure!(vsimd::hex::check_xn(s, x));
            len -= 16;
            src = src.add(16);
        }

        check_short(slice(src, len))
    }
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
mod sse2 {
    use super::*;

    use vsimd::isa::{InstructionSet, SSE2};
    use vsimd::SIMD128;

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn check(data: &[u8]) -> Result<(), Error> {
        let s = SSE2::new();
        let (mut src, mut len) = slice_parts(data);

        let end = src.add(len / 16 * 16);
        while src < end {
            let x = s.v128_load_unaligned(src);
            ensure!(vsimd::hex::check_xn(s, x));
            src = src.add(16);
        }
        len %= 16;

        if len == 0 {
            return Ok(());
        }

        check_short(slice(src, len))
    }
}
