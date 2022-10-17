use base64_simd::OutRef;
use simd_benches::{map_collect, rand_bytes, FnGroup};

use criterion::{black_box, criterion_group, criterion_main, AxisScale, PlotConfiguration};
use criterion::{BenchmarkId, Criterion, Throughput};

pub fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base64-encode");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096, 65536];
    let inputs: Vec<Vec<u8>> = map_collect(cases, rand_bytes);

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8], &mut [u8])> = &[
        ("base64-simd/auto", |src, dst| {
            let _ = base64_simd::STANDARD.encode(src, OutRef::from_slice(dst));
        }),
        ("radix64/auto", |src, dst| {
            radix64::STD.encode_slice(src, dst);
        }),
        ("base64/fallback", |src, dst| {
            let config = base64::STANDARD;
            base64::encode_config_slice(src, config, dst);
        }),
        ("base64ct/fallback", |src, dst| {
            use base64ct::Encoding;
            base64ct::Base64::encode(src, dst).unwrap();
        }),
        ("data-encoding/fallback", |src, dst| {
            data_encoding::BASE64.encode_mut(src, dst);
        }),
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());

            let mut dst: Vec<u8> = vec![0; base64_simd::STANDARD.encoded_length(src.len())];

            group.bench_with_input(id, src.as_slice(), |b, src| {
                b.iter(|| f(black_box(src), black_box(dst.as_mut_slice())))
            });
        }
    }
}

pub fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base64-decode");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096, 65536];
    let inputs: Vec<Vec<u8>> = map_collect(cases, |n| base64_simd::STANDARD.encode_type(&rand_bytes(n)));

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8], &mut [u8])> = &[
        ("base64-simd/auto", |src, dst| {
            base64_simd::STANDARD.decode(src, OutRef::from_slice(dst)).unwrap();
        }),
        ("radix64/auto", |src, dst| {
            radix64::STD.decode_slice(src, dst).unwrap();
        }),
        ("base64/fallback", |src, dst| {
            let config = base64::STANDARD;
            base64::decode_config_slice(src, config, dst).unwrap();
        }),
        ("base64ct/fallback", |src, dst| {
            use base64ct::Encoding;
            base64ct::Base64::decode(src, dst).unwrap();
        }),
        ("data-encoding/fallback", |src, dst| {
            data_encoding::BASE64.decode_mut(src, dst).unwrap();
        }),
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());

            let mut dst: Vec<u8> = vec![0; base64_simd::STANDARD.estimated_decoded_length(src.len())];

            group.bench_with_input(id, src.as_slice(), |b, src| {
                b.iter(|| f(black_box(src), black_box(dst.as_mut_slice())))
            });
        }
    }
}

pub fn bench_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("base64-check");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096, 65536];
    let inputs: Vec<Vec<u8>> = map_collect(cases, |n| base64_simd::STANDARD.encode_type(&rand_bytes(n)));

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8])> = &[
        ("base64-simd/auto", |src| {
            assert!(base64_simd::STANDARD.check(src).is_ok());
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
