use crate::pod::{align, POD};
use crate::tools::slice;
use crate::vector::{V128, V256};
use crate::SIMD256;

pub(crate) const SHUFFLE_U16X8: V128 = V128::from_bytes([
    0x01, 0x00, 0x03, 0x02, 0x05, 0x04, 0x07, 0x06, //
    0x09, 0x08, 0x0b, 0x0a, 0x0d, 0x0c, 0x0f, 0x0e, //
]);

pub(crate) const SHUFFLE_U32X4: V128 = V128::from_bytes([
    0x03, 0x02, 0x01, 0x00, 0x07, 0x06, 0x05, 0x04, //
    0x0b, 0x0a, 0x09, 0x08, 0x0f, 0x0e, 0x0d, 0x0c, //
]);

pub(crate) const SHUFFLE_U64X2: V128 = V128::from_bytes([
    0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, //
    0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x08, //
]);

pub(crate) const SHUFFLE_U16X16: V256 = SHUFFLE_U16X8.x2();

pub(crate) const SHUFFLE_U32X8: V256 = SHUFFLE_U32X4.x2();

pub(crate) const SHUFFLE_U64X4: V256 = SHUFFLE_U64X2.x2();

pub unsafe trait BSwapExt: POD {
    fn swap_single(x: Self) -> Self;
    fn swap_simd<S: SIMD256>(s: S, a: V256) -> V256;
}

unsafe impl BSwapExt for u16 {
    #[inline(always)]
    fn swap_single(x: Self) -> Self {
        x.swap_bytes()
    }

    #[inline(always)]
    fn swap_simd<S: SIMD256>(s: S, a: V256) -> V256 {
        s.u16x16_bswap(a)
    }
}

unsafe impl BSwapExt for u32 {
    #[inline(always)]
    fn swap_single(x: Self) -> Self {
        x.swap_bytes()
    }

    #[inline(always)]
    fn swap_simd<S: SIMD256>(s: S, a: V256) -> V256 {
        s.u32x8_bswap(a)
    }
}

unsafe impl BSwapExt for u64 {
    #[inline(always)]
    fn swap_single(x: Self) -> Self {
        x.swap_bytes()
    }

    #[inline(always)]
    fn swap_simd<S: SIMD256>(s: S, a: V256) -> V256 {
        s.u64x4_bswap(a)
    }
}

#[inline(always)]
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

#[inline(always)]
fn raw_align32<T: POD>(slice: &[T]) -> (SliceRawParts<T>, SliceRawParts<V256>, SliceRawParts<T>) {
    let (p, m, s) = align::<_, V256>(slice);
    let p = (p.as_ptr(), p.len());
    let m = (m.as_ptr(), m.len());
    let s = (s.as_ptr(), s.len());
    (p, m, s)
}

#[inline(always)]
pub unsafe fn bswap_fallback<T>(src: *const T, len: usize, mut dst: *mut T)
where
    T: BSwapExt,
{
    unroll_ptr(src, len, 8, |src| {
        let x = src.read();
        let y = BSwapExt::swap_single(x);
        dst.write(y);
        dst = dst.add(1);
    });
}

#[inline(always)]
pub unsafe fn bswap_simd<S: SIMD256, T>(s: S, src: *const T, len: usize, mut dst: *mut T)
where
    T: BSwapExt,
{
    let (prefix, middle, suffix) = raw_align32(slice(src, len));

    {
        let (src, len) = prefix;
        bswap_fallback(src, len, dst);
        dst = dst.add(len);
    }

    {
        let (src, len) = middle;
        unroll_ptr(src, len, 8, |src| {
            let x = s.v256_load(src.cast());
            let y = <T as BSwapExt>::swap_simd(s, x);
            s.v256_store_unaligned(dst.cast(), y);
            dst = dst.add(8);
        });
    }

    {
        let (src, len) = suffix;
        bswap_fallback(src, len, dst);
    }
}
