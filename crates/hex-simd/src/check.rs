use crate::sa_hex::{self, unhex};

use simd_abstraction::scalar::Bytes32;
use simd_abstraction::tools::read;
use simd_abstraction::traits::{SimdLoad, SIMD256};

#[inline]
pub fn check_fallback(data: &[u8]) -> bool {
    #[inline(always)]
    unsafe fn check_unroll1(n: usize, src: *const u8) -> bool {
        let mut i = 0;
        let mut ans = 0;
        while i < n {
            ans |= unhex(read(src, i));
            i += 1;
        }
        ans != 0xff
    }
    #[inline(always)]
    unsafe fn check_unroll4(n: usize, src: *const u8) -> bool {
        let mut i = 0;
        while i < n {
            let y1 = unhex(read(src, i));
            let y2 = unhex(read(src, i + 1));
            let y3 = unhex(read(src, i + 2));
            let y4 = unhex(read(src, i + 3));
            if y1 | y2 | y3 | y4 == 0xff {
                return false;
            }
            i += 4;
        }
        true
    }

    let n = data.len();
    let src = data.as_ptr();
    unsafe {
        let n1 = n & 3;
        let n4 = n - n1;
        if n4 > 0 && !check_unroll4(n4, src) {
            return false;
        }
        check_unroll1(n1, src.add(n4))
    }
}

#[inline]
pub fn check_simd<S: SIMD256>(s: S, data: &[u8]) -> bool {
    let (prefix, chunks, suffix) = unsafe { data.align_to::<Bytes32>() };
    if !check_fallback(prefix) {
        return false;
    }
    for chunk in chunks {
        if !sa_hex::check_u8x32(s, s.load(chunk)) {
            return false;
        }
    }
    if !check_fallback(suffix) {
        return false;
    }
    true
}
