use rand::RngCore;

pub fn rand_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0; len];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

pub type FnGroup<T> = [(&'static str, T)];

pub fn map_collect<T, U, C>(iter: impl IntoIterator<Item = T>, f: impl FnMut(T) -> U) -> C
where
    C: FromIterator<U>,
{
    iter.into_iter().map(f).collect()
}

pub mod faster_hex {
    #[inline]
    pub fn hex_check(src: &[u8]) -> bool {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if cfg!(target_feature = "sse4.1") {
                return unsafe { ::faster_hex::hex_check_sse(src) };
            }

            #[cfg(feature = "detect")]
            if is_x86_feature_detected!("sse4.1") {
                return unsafe { ::faster_hex::hex_check_sse(src) };
            }
        }
        ::faster_hex::hex_check_fallback(src)
    }
}

#[allow(clippy::needless_lifetimes)]
#[cfg(feature = "parallel")]
#[inline]
#[must_use]
pub fn par_base64_encode<'s, 'd>(src: &'s [u8], dst: &'d mut [u8]) -> &'d mut [u8] {
    use base64_simd::AsOut;
    use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
    use rayon::slice::{ParallelSlice, ParallelSliceMut};

    let p = rayon::current_num_threads();
    let b = src.len() / 3;
    if p < 2 || b < p {
        return base64_simd::STANDARD.encode(src, dst.as_out());
    }

    let encoded_len = base64_simd::STANDARD.encoded_length(src.len());
    let dst = &mut dst[..encoded_len];

    let chunks = (b + p) / p;

    let src_chunks = src.par_chunks(chunks * 3);
    let dst_chunks = dst.par_chunks_mut(chunks * 4);

    src_chunks.zip(dst_chunks).for_each(|(s, d)| {
        if s.len() % 3 == 0 {
            let _ = base64_simd::STANDARD_NO_PAD.encode(s, d.as_out());
        } else {
            let _ = base64_simd::STANDARD.encode(s, d.as_out());
        }
    });

    dst
}

#[cfg(test)]
mod tests {
    /// cargo test -p simd-benches --lib --release --features parallel
    #[cfg(feature = "parallel")]
    #[test]
    fn test_par_base64_encode() {
        use super::*;
        use base64_simd::AsOut;
        let mut buf1 = vec![0; 100_000];
        let mut buf2 = vec![0; 100_000];
        for n in 0..50_000 {
            let src = rand_bytes(n);
            let ans1 = par_base64_encode(&src, &mut buf1);
            let ans2 = base64_simd::STANDARD.encode(&src, buf2.as_out());
            assert!(ans1 == ans2, "n = {n}");
        }
    }
}
