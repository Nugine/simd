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

// #[inline]
// #[must_use]
// pub fn is_ascii(src: &[u8]) -> bool {
//     #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
//     {
//         is_ascii_sse2(src)
//     }
//     #[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
//     {
//         src.is_ascii()
//     }
// }

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline]
#[must_use]
pub fn is_ascii_sse2(src: &[u8]) -> bool {
    use core::arch::x86_64::*;

    macro_rules! ensure {
        ($cond:expr) => {
            if !$cond {
                return false;
            }
        };
    }

    #[inline(always)]
    unsafe fn loadu<T>(p: *const u8) -> T {
        p.cast::<T>().read_unaligned()
    }

    #[inline(always)]
    fn check4(x: u32) -> bool {
        (x & 0x8080_8080) == 0
    }

    #[inline(always)]
    fn check8(x: u64) -> bool {
        (x & 0x8080_8080_8080_8080) == 0
    }

    #[inline(always)]
    unsafe fn check16(x: __m128i) -> bool {
        _mm_movemask_epi8(x) as u32 as u16 == 0
    }

    #[inline(always)]
    unsafe fn or(a: __m128i, b: __m128i) -> __m128i {
        _mm_or_si128(a, b)
    }

    /// len in 0..=8
    #[inline(always)]
    unsafe fn check_tiny(mut src: *const u8, mut len: usize) -> bool {
        if len == 8 {
            return check8(loadu(src));
        }
        if len >= 4 {
            ensure!(check4(loadu(src)));
            src = src.add(4);
            len -= 4;
        }
        {
            let mut acc: u8 = 0;
            let end = src.add(len);
            for _ in 0..3 {
                if src < end {
                    acc |= src.read();
                    src = src.add(1);
                }
            }
            acc < 0x80
        }
    }

    /// len in 9..=16
    #[inline(always)]
    unsafe fn check_short(src: *const u8, len: usize) -> bool {
        let x1: u64 = loadu(src);
        let x2: u64 = loadu(src.add(len - 8));
        check8(x1 | x2)
    }

    /// len in 17..64
    #[inline(always)]
    unsafe fn check_medium(src: *const u8, len: usize) -> bool {
        let mut x: __m128i = loadu(src);
        if len >= 32 {
            x = or(x, loadu(src.add(16)));
        }
        if len >= 48 {
            x = or(x, loadu(src.add(32)));
        }
        x = or(x, loadu(src.add(len - 16)));
        check16(x)
    }

    /// len >= 64
    #[inline(always)]
    unsafe fn check_long(mut src: *const u8, mut len: usize) -> bool {
        let end = src.add(len / 64 * 64);
        while src < end {
            let x: [__m128i; 4] = loadu(src);
            ensure!(check16(or(or(x[0], x[1]), or(x[2], x[3]))));
            src = src.add(64);
        }
        len %= 64;
        if len != 0 {
            ensure!(check_medium(src, len))
        }
        true
    }

    unsafe {
        let len = src.len();
        let src = src.as_ptr();

        if len <= 8 {
            check_tiny(src, len)
        } else if len <= 16 {
            check_short(src, len)
        } else if len < 64 {
            check_medium(src, len)
        } else {
            check_long(src, len)
        }
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
