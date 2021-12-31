use criterion::{black_box, criterion_group, criterion_main};
use criterion::{Bencher, BenchmarkId, Criterion, Throughput};
use rand::RngCore;

use hex_simd::{AsciiCase, OutBuf};

fn gen_hex_chars(len: usize) -> Vec<u8> {
    let chars = b"0123456789abcdef";
    let mut buf = vec![0; len];
    rand::thread_rng().fill_bytes(&mut buf);
    let to_hex = |x: &mut u8| *x = chars[(*x % 16) as usize];
    buf.iter_mut().for_each(to_hex);
    buf
}

fn gen_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0; len];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

pub fn bench_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex-simd-check");

    let inputs: Vec<Vec<u8>> = [16, 32, 64, 256, 1024, 4096]
        .iter()
        .copied()
        .map(gen_hex_chars)
        .collect();

    #[allow(clippy::type_complexity)]
    let functions: &[(&str, fn(&mut Bencher, &[u8]))] = &[
        #[cfg(target_feature = "avx2")]
        ("hex-simd/avx2", |b, src| {
            b.iter(|| unsafe { assert!(hex_simd::arch::x86::avx2::check(black_box(src))) })
        }),
        #[cfg(target_feature = "sse4.1")]
        ("hex-simd/sse4.1", |b, src| {
            b.iter(|| unsafe { assert!(hex_simd::arch::x86::sse41::check(black_box(src))) })
        }),
        #[cfg(target_feature = "sse4.1")]
        ("faster-hex/sse4.1", |b, src| {
            b.iter(|| unsafe { assert!(faster_hex::hex_check_sse(black_box(src))) })
        }),
        ("hex-simd/fallback", |b, src| {
            b.iter(|| assert!(hex_simd::fallback::check(black_box(src))))
        }),
        ("faster-hex/fallback", |b, src| {
            b.iter(|| assert!(faster_hex::hex_check_fallback(black_box(src))))
        }),
    ];

    for &(name, f) in functions {
        for input in &inputs {
            group.throughput(Throughput::Bytes(input.len() as u64));
            let id = BenchmarkId::new(name, input.len());
            group.bench_with_input(id, input.as_slice(), f);
        }
    }
}

pub fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex-simd-decode");

    let inputs: Vec<Vec<u8>> = [16, 32, 64, 256, 1024, 4096]
        .iter()
        .copied()
        .map(gen_hex_chars)
        .collect();

    #[allow(clippy::type_complexity)]
    let functions: &[(&str, fn(&mut Bencher, &[u8], &mut [u8]))] = &[
        #[cfg(target_feature = "avx2")]
        ("hex-simd/avx2", |b, src, dst| {
            b.iter(|| unsafe {
                let (src, dst) = (black_box(src), black_box(OutBuf::from_slice_mut(dst)));
                hex_simd::arch::x86::avx2::decode(src, dst).unwrap();
            })
        }),
        #[cfg(target_feature = "sse4.1")]
        ("hex-simd/sse4.1", |b, src, dst| {
            b.iter(|| unsafe {
                let (src, dst) = (black_box(src), black_box(OutBuf::from_slice_mut(dst)));
                hex_simd::arch::x86::sse41::decode(src, dst).unwrap();
            })
        }),
        #[cfg(target_feature = "avx2")]
        ("faster-hex/avx2", |b, src, dst| {
            b.iter(|| {
                faster_hex::hex_decode(black_box(src), black_box(dst)).unwrap();
            })
        }),
        ("hex-simd/fallback", |b, src, dst| {
            b.iter(|| {
                let (src, dst) = (black_box(src), black_box(OutBuf::from_slice_mut(dst)));
                hex_simd::fallback::decode(src, dst).unwrap();
            })
        }),
        ("faster-hex/fallback", |b, src, dst| {
            b.iter(|| {
                assert!(faster_hex::hex_check_fallback(src));
                faster_hex::hex_decode_fallback(black_box(src), black_box(dst));
            })
        }),
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());
            let mut dst = vec![0; src.len() / 2];
            group.bench_with_input(id, src.as_slice(), |b, src| f(b, src, dst.as_mut_slice()));
        }
    }
}

pub fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex-simd-encode");

    let inputs: Vec<Vec<u8>> = [16, 32, 64, 256, 1024, 4096]
        .iter()
        .copied()
        .map(gen_bytes)
        .collect();

    #[allow(clippy::type_complexity)]
    let functions: &[(&str, fn(&mut Bencher, &[u8], &mut [u8]))] = &[
        #[cfg(target_feature = "avx2")]
        ("hex-simd/avx2", |b, src, dst| {
            b.iter(|| unsafe {
                let (src, dst) = (black_box(src), black_box(OutBuf::from_slice_mut(dst)));
                hex_simd::arch::x86::avx2::encode(src, dst, AsciiCase::Lower).unwrap();
            })
        }),
        #[cfg(target_feature = "sse4.1")]
        ("hex-simd/sse4.1", |b, src, dst| {
            b.iter(|| unsafe {
                let (src, dst) = (black_box(src), black_box(OutBuf::from_slice_mut(dst)));
                hex_simd::arch::x86::sse41::encode(src, dst, AsciiCase::Lower).unwrap();
            })
        }),
        #[cfg(target_feature = "avx2")]
        ("faster-hex/avx2", |b, src, dst| {
            b.iter(|| {
                faster_hex::hex_encode(black_box(src), black_box(dst)).unwrap();
            })
        }),
        ("hex-simd/fallback", |b, src, dst| {
            b.iter(|| {
                let (src, dst) = (black_box(src), black_box(OutBuf::from_slice_mut(dst)));
                hex_simd::fallback::encode(src, dst, AsciiCase::Lower).unwrap();
            })
        }),
        ("faster-hex/fallback", |b, src, dst| {
            b.iter(|| {
                faster_hex::hex_encode_fallback(black_box(src), black_box(dst));
            })
        }),
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());
            let mut dst: Vec<u8> = vec![0; src.len() * 2];
            group.bench_with_input(id, src.as_slice(), |b, src| f(b, src, dst.as_mut_slice()));
        }
    }
}

criterion_group!(benches, bench_check, bench_decode, bench_encode);
criterion_main!(benches);
