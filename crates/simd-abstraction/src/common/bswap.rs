#![allow(clippy::missing_safety_doc)]

use crate::isa::{SimdLoad, SIMD256};
use crate::scalar::{align32, Bytes16, Bytes32, Scalar};

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "wasm32"))]
pub(crate) const SHUFFLE_U16X8: &Bytes16 = &Bytes16([
    0x01, 0x00, 0x03, 0x02, 0x05, 0x04, 0x07, 0x06, //
    0x09, 0x08, 0x0b, 0x0a, 0x0d, 0x0c, 0x0f, 0x0e, //
]);

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "wasm32"))]
pub(crate) const SHUFFLE_U32X4: &Bytes16 = &Bytes16([
    0x03, 0x02, 0x01, 0x00, 0x07, 0x06, 0x05, 0x04, //
    0x0b, 0x0a, 0x09, 0x08, 0x0f, 0x0e, 0x0d, 0x0c, //
]);

pub(crate) const SHUFFLE_U64X2: &Bytes16 = &Bytes16([
    0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, //
    0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x08, //
]);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) const SHUFFLE_U16X16: &Bytes32 = &Bytes32::double(SHUFFLE_U16X8.0);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) const SHUFFLE_U32X8: &Bytes32 = &Bytes32::double(SHUFFLE_U32X4.0);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) const SHUFFLE_U64X4: &Bytes32 = &Bytes32::double(SHUFFLE_U64X2.0);

unsafe fn unroll_ptr<T>(
    mut src: *const T,
    len: usize,
    chunk_size: usize,
    mut f: impl FnMut(*const T),
) {
    let chunks_end = src.add(len / chunk_size * chunk_size);
    let end = src.add(len);

    while src < chunks_end {
        for _ in 0..chunk_size {
            f(src);
            src = src.add(1);
        }
    }

    while src < end {
        f(src);
        src = src.add(1);
    }
}

type SliceRawParts<T> = (*const T, usize);

fn raw_align32<T: Scalar>(
    slice: &[T],
) -> (SliceRawParts<T>, SliceRawParts<Bytes32>, SliceRawParts<T>) {
    let (p, m, s) = align32(slice);
    let p = (p.as_ptr(), p.len());
    let m = (m.as_ptr(), m.len());
    let s = (s.as_ptr(), s.len());
    (p, m, s)
}

#[inline]
pub unsafe fn bswap_u32_raw_fallback(src: *const u32, len: usize, mut dst: *mut u32) {
    unroll_ptr(src, len, 8, |src| {
        dst.write(src.read().swap_bytes());
        dst = dst.add(1);
    })
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn bswap_u32_raw_simd<S: SIMD256>(s: S, src: *const u32, len: usize, mut dst: *mut u32) {
    let (prefix, middle, suffix) = raw_align32(core::slice::from_raw_parts(src, len));

    {
        let (src, len) = prefix;
        bswap_u32_raw_fallback(src, len, dst);
        dst = dst.add(len)
    }

    {
        let (src, len) = middle;
        unroll_ptr(src, len, 8, |src| {
            let x = s.load(&*src);
            let y = s.u32x8_bswap(x);
            s.v256_store_unaligned(dst.cast(), y);
            dst = dst.add(8);
        })
    }

    {
        let (src, len) = suffix;
        bswap_u32_raw_fallback(src, len, dst);
    }
}

pub mod multiversion {
    use super::*;

    crate::simd_dispatch! (
        name        = bswap_u32_raw,
        signature   = fn(src: *const u32, len: usize, dst: *mut u32) -> (),
        fallback    = {bswap_u32_raw_fallback},
        simd        = {bswap_u32_raw_simd},
        safety      = {unsafe},
    );
}
