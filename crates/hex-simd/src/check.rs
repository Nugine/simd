use crate::Error;

use vsimd::hex::unhex;
use vsimd::isa::AVX2;
use vsimd::{matches_isa, SIMD256};

#[inline(always)]
unsafe fn check_short(mut src: *const u8, len: usize) -> Result<(), Error> {
    // FIXME:
    // The later version triggers incorrect auto-vectorization when avx2 is enabled.
    // https://github.com/Nugine/simd/issues/14
    // https://github.com/rust-lang/rust/issues/102709
    //

    if cfg!(target_feature = "avx2") {
        let end = src.add(len);
        while src < end {
            let x = src.read();
            ensure!(unhex(x) != 0xff);
            src = src.add(1);
        }
    } else {
        let mut flag = 0;
        let end = src.add(len);
        while src < end {
            let x = src.read();
            flag |= unhex(x);
            src = src.add(1);
        }
        ensure!(flag != 0xff);
    }

    Ok(())
}

#[inline(always)]
pub unsafe fn check_fallback(src: *const u8, len: usize) -> Result<(), Error> {
    check_short(src, len)
}

#[inline(always)]
pub unsafe fn check_simd<S: SIMD256>(s: S, mut src: *const u8, mut len: usize) -> Result<(), Error> {
    if matches_isa!(S, AVX2) {
        if len == 16 {
            let x = s.v128_load_unaligned(src);
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
    } else {
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
    }

    check_short(src, len)
}
