use hex_simd::{AsciiCase, OutRef};
use simd_benches::{map_collect, rand_bytes, FnGroup};

use criterion::{black_box, criterion_group, criterion_main, AxisScale, PlotConfiguration};
use criterion::{BenchmarkId, Criterion, Throughput};

fn gen_hex_chars(len: usize) -> Vec<u8> {
    let mut buf = rand_bytes(len);
    let chars = b"0123456789abcdef";
    let to_hex = |x: &mut u8| *x = chars[(*x % 16) as usize];
    buf.iter_mut().for_each(to_hex);
    buf
}

pub fn bench_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex-check");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096];
    let inputs: Vec<Vec<u8>> = map_collect(cases, gen_hex_chars);

    let functions: &FnGroup<fn(&[u8])> = &[
        ("hex-simd/auto", |src: &[u8]| {
            assert!(hex_simd::check(src).is_ok()); //
        }),
        #[cfg(target_feature = "sse4.1")]
        ("faster-hex/sse4.1", |src: &[u8]| unsafe {
            assert!(faster_hex::hex_check_sse(src))
        }),
        ("faster-hex/fallback", |src: &[u8]| {
            assert!(faster_hex::hex_check_fallback(src))
        }),
    ];

    for &(name, f) in functions {
        for input in &inputs {
            group.throughput(Throughput::Bytes(input.len() as u64));
            let id = BenchmarkId::new(name, input.len());

            group.bench_with_input(id, input.as_slice(), |b, src| b.iter(|| f(black_box(src))));
        }
    }
}

pub fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex-decode");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096];
    let inputs: Vec<Vec<u8>> = map_collect(cases, gen_hex_chars);

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8], &mut [u8])> = &[
        ("hex-simd/auto", |src, dst| {
            hex_simd::decode(src, OutRef::from_slice(dst)).unwrap();
        }),
        ("faster-hex/auto-direct", |src, dst| {
            faster_hex::hex_decode(src, dst).unwrap();
        }),
        ("faster-hex/fallback", |src, dst| {
            assert!(faster_hex::hex_check_fallback(src));
            faster_hex::hex_decode_fallback(src, dst);
        }),
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());

            let mut dst = vec![0; src.len() / 2];

            group.bench_with_input(id, src.as_slice(), |b, src| {
                b.iter(|| f(black_box(src), black_box(dst.as_mut_slice())))
            });
        }
    }
}

pub fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex-encode");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096];
    let inputs: Vec<Vec<u8>> = map_collect(cases, rand_bytes);

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8], &mut [u8])> = &[
        ("hex-simd/auto", |src, dst| {
            let _ = hex_simd::encode(src, OutRef::from_slice(dst), AsciiCase::Lower);
        }),
        ("faster-hex/auto-direct", |src, dst| {
            faster_hex::hex_encode(src, dst).unwrap();
        }),
        ("faster-hex/fallback", |src, dst| {
            faster_hex::hex_encode_fallback(src, dst);
        }),
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());

            let mut dst: Vec<u8> = vec![0; src.len() * 2];

            group.bench_with_input(id, src.as_slice(), |b, src| {
                b.iter(|| f(black_box(src), black_box(dst.as_mut_slice())))
            });
        }
    }
}

criterion_group!(benches, bench_check, bench_decode, bench_encode);
criterion_main!(benches);
