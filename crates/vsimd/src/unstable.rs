use core::ops::Not;
use core::simd::*;

#[inline(always)]
pub fn splat<T, const N: usize>(x: T) -> Simd<T, N>
where
    T: SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    Simd::splat(x)
}

#[inline]
#[must_use]
pub fn is_ascii(src: &[u8]) -> bool {
    if cfg!(any(
        all(target_arch = "x86_64", target_feature = "sse2"),
        all(target_arch = "aarch64", target_feature = "neon"),
    )) {
        is_ascii_simd(src)
    } else {
        <[u8]>::is_ascii(src)
    }
}

#[inline]
fn is_ascii_simd(src: &[u8]) -> bool {
    macro_rules! ensure {
        ($cond:expr) => {
            if !$cond {
                return false;
            }
        };
    }

    let (prefix, middle, suffix) = src.as_simd::<16>();

    ensure!(is_ascii_short(prefix));

    let mut iter = middle.array_chunks::<4>();
    for chunks in &mut iter {
        let x = array_reduce(chunks, |acc, x| acc | x);
        ensure!(is_ascii_u8x16(x));
    }

    for &x in iter.remainder() {
        ensure!(is_ascii_u8x16(x));
    }

    is_ascii_short(suffix)
}

#[inline(always)]
fn is_ascii_u8x16(x: u8x16) -> bool {
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        return x.cast::<i8>().simd_lt(i8x16::splat(0)).any().not();
    }
    if cfg!(target_arch = "aarch64") {
        return x.cast::<i8>().reduce_min() >= 0;
    }
    unreachable!() // suboptimial codegen on other archs
}

#[inline(always)]
fn is_ascii_short(src: &[u8]) -> bool {
    // TODO: word-size simd
    src.iter().copied().all(|b| b.is_ascii())
}

#[allow(clippy::needless_range_loop)]
#[inline(always)]
fn array_reduce<T: Copy, const N: usize>(arr: &[T; N], mut f: impl FnMut(T, T) -> T) -> T {
    const { assert!(N > 0) };

    let mut acc = arr[0];
    for i in 1..N {
        acc = f(acc, arr[i]);
    }
    acc
}
