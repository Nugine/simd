use crate::traits::InstructionSet;

#[allow(clippy::missing_safety_doc)]
pub unsafe trait CRC32<const P: u32>: InstructionSet {
    fn crc32_u8(self, crc: u32, value: u8) -> u32;
    fn crc32_u16(self, crc: u32, value: u16) -> u32;
    fn crc32_u32(self, crc: u32, value: u32) -> u32;
    fn crc32_u64(self, crc: u32, value: u64) -> u32;
}

pub const POLYNOMIAL_CRC32_IEEE: u32 = 0x04C11DB7;
pub const POLYNOMIAL_CRC32C: u32 = 0x1EDC6F41;

#[inline]
pub fn compute<S, const P: u32>(s: S, init: u32, data: &[u8]) -> u32
where
    S: CRC32<P>,
{
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
