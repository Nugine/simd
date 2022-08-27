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

pub unsafe trait BSwapExt: Scalar {
    fn swap_single(x: Self) -> Self;
    fn swap_simd<S: SIMD256>(s: S, a: S::V256) -> S::V256;
}

unsafe impl BSwapExt for u16 {
    #[inline(always)]
    fn swap_single(x: Self) -> Self {
        x.swap_bytes()
    }

    #[inline(always)]
    fn swap_simd<S: SIMD256>(s: S, a: S::V256) -> S::V256 {
        s.u16x16_bswap(a)
    }
}

unsafe impl BSwapExt for u32 {
    #[inline(always)]
    fn swap_single(x: Self) -> Self {
        x.swap_bytes()
    }

    #[inline(always)]
    fn swap_simd<S: SIMD256>(s: S, a: S::V256) -> S::V256 {
        s.u32x8_bswap(a)
    }
}

unsafe impl BSwapExt for u64 {
    #[inline(always)]
    fn swap_single(x: Self) -> Self {
        x.swap_bytes()
    }

    #[inline(always)]
    fn swap_simd<S: SIMD256>(s: S, a: S::V256) -> S::V256 {
        s.u64x4_bswap(a)
    }
}

unsafe fn unroll_ptr<T>(mut src: *const T, len: usize, chunk_size: usize, mut f: impl FnMut(*const T)) {
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

fn raw_align32<T: Scalar>(slice: &[T]) -> (SliceRawParts<T>, SliceRawParts<Bytes32>, SliceRawParts<T>) {
    let (p, m, s) = align32(slice);
    let p = (p.as_ptr(), p.len());
    let m = (m.as_ptr(), m.len());
    let s = (s.as_ptr(), s.len());
    (p, m, s)
}

#[inline]
pub unsafe fn bswap_fallback<T>(src: *const T, len: usize, mut dst: *mut T)
where
    T: BSwapExt,
{
    unroll_ptr(src, len, 8, |src| {
        let x = src.read();
        let y = BSwapExt::swap_single(x);
        dst.write(y);
        dst = dst.add(1);
    })
}

#[inline]
pub unsafe fn bswap_simd<S: SIMD256, T>(s: S, src: *const T, len: usize, mut dst: *mut T)
where
    T: BSwapExt,
{
    let (prefix, middle, suffix) = raw_align32(core::slice::from_raw_parts(src, len));

    {
        let (src, len) = prefix;
        bswap_fallback(src, len, dst);
        dst = dst.add(len)
    }

    {
        let (src, len) = middle;
        unroll_ptr(src, len, 8, |src| {
            let x = s.load(&*src);
            let y = <T as BSwapExt>::swap_simd(s, x);
            s.v256_store_unaligned(dst.cast(), y);
            dst = dst.add(8);
        })
    }

    {
        let (src, len) = suffix;
        bswap_fallback(src, len, dst);
    }
}
