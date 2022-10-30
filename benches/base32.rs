use base32_simd::AsOut;
use simd_benches::{map_collect, rand_bytes, FnGroup};

use criterion::{black_box, criterion_group, criterion_main, AxisScale, PlotConfiguration};
use criterion::{BenchmarkId, Criterion, Throughput};

pub fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base32-encode");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096, 65536];
    let inputs: Vec<Vec<u8>> = map_collect(cases, rand_bytes);

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8], &mut [u8])> = &[
        ("base32-simd/auto", |src, dst| {
            let _ = base32_simd::BASE32.encode(src, dst.as_out()); //
        }),
        ("base32ct/fallback", |src, dst| {
            use base32ct::Encoding;
            base32ct::Base32Upper::encode(src, dst).unwrap();
        }),
        ("data-encoding/fallback", |src, dst| {
            data_encoding::BASE32.encode_mut(src, dst);
        }),
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());

            let mut dst: Vec<u8> = vec![0; base32_simd::BASE32.encoded_length(src.len())];

            group.bench_with_input(id, src.as_slice(), |b, src| {
                b.iter(|| f(black_box(src), black_box(dst.as_mut_slice())))
            });
        }
    }
}

pub fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base32-decode");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096, 65536];
    let inputs: Vec<Vec<u8>> = map_collect(cases, |n| base32_simd::BASE32.encode_type(&rand_bytes(n)));

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8], &mut [u8])> = &[
        ("base32-simd/auto", |src, dst| {
            base32_simd::BASE32.decode(src, dst.as_out()).unwrap(); //
        }),
        ("base32ct/fallback", |src, dst| {
            use base32ct::Encoding;
            base32ct::Base32Upper::decode(src, dst).unwrap();
        }),
        ("data-encoding/fallback", |src, dst| {
            data_encoding::BASE32.decode_mut(src, dst).unwrap();
        }),
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());

            let mut dst: Vec<u8> = vec![0; base32_simd::BASE32.estimated_decoded_length(src.len())];

            group.bench_with_input(id, src.as_slice(), |b, src| {
                b.iter(|| f(black_box(src), black_box(dst.as_mut_slice())))
            });
        }
    }
}

pub fn bench_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("base32-check");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096, 65536];
    let inputs: Vec<Vec<u8>> = map_collect(cases, |n| base32_simd::BASE32.encode_type(&rand_bytes(n)));

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8])> = &[
        ("base32-simd/auto", |src| {
            assert!(base32_simd::BASE32.check(src).is_ok());
        }), //
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());

            group.bench_with_input(id, src.as_slice(), |b, src| b.iter(|| f(black_box(src))));
        }
    }
}

criterion_group!(benches, bench_encode, bench_decode, bench_check);
criterion_main!(benches);
