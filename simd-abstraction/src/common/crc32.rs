use crate::traits::{CRC32, POLYNOMIAL_CRC32C, POLYNOMIAL_CRC32_IEEE};

#[inline]
pub fn compute<S: CRC32<P>, const P: u32>(s: S, init: u32, data: &[u8]) -> u32 {
    let mut crc = !init;

    let (prefix, middle, suffix) = unsafe { data.align_to::<u64>() };

    crc = crc32_inner_u8_slice(s, crc, prefix);
    crc = crc32_inner_u64_slice_unroll8(s, crc, middle);
    crc = crc32_inner_u8_slice(s, crc, suffix);

    !crc
}

#[inline(always)]
fn crc32_inner_u8_slice<S: CRC32<P>, const P: u32>(s: S, init: u32, data: &[u8]) -> u32 {
    data.iter().fold(init, |crc, &value| s.crc32_u8(crc, value))
}

#[inline(always)]
fn crc32_inner_u64_slice<S: CRC32<P>, const P: u32>(s: S, init: u32, data: &[u64]) -> u32 {
    let fold = |crc, &value| s.crc32_u64(crc, value);
    data.iter().fold(init, fold)
}

#[inline(always)]
fn crc32_inner_u64_slice_unroll8<S: CRC32<P>, const P: u32>(s: S, init: u32, data: &[u64]) -> u32 {
    let mut iter = data.chunks_exact(8);
    let fold = |crc, chunk| crc32_inner_u64_slice(s, crc, chunk);
    let crc = iter.by_ref().fold(init, fold);
    crc32_inner_u64_slice(s, crc, iter.remainder())
}

#[inline]
pub fn compute_crc32_ieee<S: CRC32<POLYNOMIAL_CRC32_IEEE>>(s: S, init: u32, data: &[u8]) -> u32 {
    compute::<S, POLYNOMIAL_CRC32_IEEE>(s, init, data)
}

#[inline]
pub fn compute_crc32c<S: CRC32<POLYNOMIAL_CRC32C>>(s: S, init: u32, data: &[u8]) -> u32 {
    compute::<S, POLYNOMIAL_CRC32C>(s, init, data)
}
