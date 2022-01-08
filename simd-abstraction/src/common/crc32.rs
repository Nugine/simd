use crate::traits::{CRC32, POLYNOMIAL_CRC32C, POLYNOMIAL_CRC32_IEEE};

#[inline]
pub fn compute<S: CRC32<P>, const P: u32>(s: S, init: u32, data: &[u8]) -> u32 {
    let mut crc = !init;

    let (prefix, middle, suffix) = unsafe { data.align_to::<u64>() };

    let fold_u8 = |crc, value| s.crc32_u8(crc, value);
    crc = fold_copied(prefix, crc, fold_u8);

    crc = {
        let fold_u64 = |crc, value| s.crc32_u64(crc, value);
        let fold_chunk = |crc, chunk: &[u64]| fold_copied(chunk, crc, fold_u64);
        let mut iter = middle.chunks_exact(8);
        let crc = iter.by_ref().fold(crc, fold_chunk);
        fold_copied(iter.remainder(), crc, fold_u64)
    };

    crc = fold_copied(suffix, crc, fold_u8);

    !crc
}

fn fold_copied<T: Copy, B>(slice: &[T], init: B, f: impl Fn(B, T) -> B) -> B {
    slice.iter().copied().fold(init, f)
}

#[inline]
pub fn compute_crc32_ieee<S: CRC32<POLYNOMIAL_CRC32_IEEE>>(s: S, init: u32, data: &[u8]) -> u32 {
    compute::<S, POLYNOMIAL_CRC32_IEEE>(s, init, data)
}

#[inline]
pub fn compute_crc32c<S: CRC32<POLYNOMIAL_CRC32C>>(s: S, init: u32, data: &[u8]) -> u32 {
    compute::<S, POLYNOMIAL_CRC32C>(s, init, data)
}
