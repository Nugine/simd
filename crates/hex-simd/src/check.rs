use crate::Error;

use vsimd::hex::unhex;
use vsimd::SIMD256;

#[inline]
pub fn check_fallback(data: &[u8]) -> Result<(), Error> {
    let mut iter = data.chunks_exact(4);
    for chunk in &mut iter {
        let y1 = unhex(chunk[0]);
        let y2 = unhex(chunk[1]);
        let y3 = unhex(chunk[2]);
        let y4 = unhex(chunk[3]);
        ensure!((y1 | y2 | y3 | y4) != 0xff);
    }
    let flag = iter.remainder().iter().fold(0, |acc, &x| acc | unhex(x));
    ensure!(flag != 0xff);
    Ok(())
}

#[inline]
pub fn check_simd<S: SIMD256>(s: S, data: &[u8]) -> Result<(), Error> {
    unsafe {
        let (mut src, mut len) = (data.as_ptr(), data.len());

        while len >= 32 {
            let x = s.v256_load_unaligned(src);
            let is_ascii = vsimd::hex::check_ascii32(s, x);
            ensure!(is_ascii);
            len -= 32;
            src = src.add(32);
        }

        if len >= 16 {
            let x = s.v128_load_unaligned(src);
            let is_ascii = vsimd::hex::check_ascii16(s, x);
            ensure!(is_ascii);
            len -= 16;
            src = src.add(16);
        }

        let suffix = core::slice::from_raw_parts(src, len);
        check_fallback(suffix)
    }
}
