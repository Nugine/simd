use base64_simd::AsOut;
use simd_benches::{map_collect, rand_bytes, FnGroup};

use criterion::{black_box, criterion_group, criterion_main, AxisScale, PlotConfiguration};
use criterion::{BenchmarkId, Criterion, Throughput};

pub fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base64-encode");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let inputs: Vec<Vec<u8>> = {
        let mut cases = vec![16, 32, 64, 256, 1024, 4096, 64 * 1024];
        if cfg!(feature = "parallel") {
            cases.extend_from_slice(&[256 * 1024, 512 * 1024, 1024 * 1024]);
        }
        map_collect(cases, rand_bytes)
    };

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8], &mut [u8])> = &[
        ("base64-simd/auto", |src, dst| {
            let _ = base64_simd::STANDARD.encode(src, dst.as_out());
        }),
        #[cfg(feature = "parallel")]
        ("base64-simd/parallel", |src, dst| {
            let _ = base64_simd::STANDARD.par_encode(src, dst.as_out());
        }),
        ("radix64/auto", |src, dst| {
            radix64::STD.encode_slice(src, dst);
        }),
        ("base64/fallback", |src, dst| {
            use base64::Engine as _;
            base64::prelude::BASE64_STANDARD.encode_slice(src, dst).unwrap();
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
    let inputs: Vec<Vec<u8>> = map_collect(cases, |n| base64_simd::STANDARD.encode_type(rand_bytes(n)));

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8], &mut [u8])> = &[
        ("base64-simd/auto", |src, dst| {
            base64_simd::STANDARD.decode(src, dst.as_out()).unwrap();
        }),
        ("radix64/auto", |src, dst| {
            radix64::STD.decode_slice(src, dst).unwrap();
        }),
        ("base64/fallback", |src, dst| {
            use base64::Engine as _;
            base64::prelude::BASE64_STANDARD.decode_slice(src, dst).unwrap();
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
    let inputs: Vec<Vec<u8>> = map_collect(cases, |n| base64_simd::STANDARD.encode_type(rand_bytes(n)));

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

pub fn bench_forgiving_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("base64-forgiving-decode");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [16, 32, 64, 256, 1024, 4096, 65536];
    let inputs: Vec<Vec<u8>> = map_collect(cases, |n| base64_simd::STANDARD.encode_type(rand_bytes(n)));

    #[allow(clippy::type_complexity)]
    let functions: &FnGroup<fn(&[u8], &mut [u8])> = &[
        ("base64-simd/auto", |src, dst| {
            base64_simd::forgiving_decode(src, dst.as_out()).unwrap();
        }), //
    ];

    for &(name, f) in functions {
        for src in &inputs {
            group.throughput(Throughput::Bytes(src.len() as u64));
            let id = BenchmarkId::new(name, src.len());

            let mut dst: Vec<u8> = vec![0; src.len()];

            group.bench_with_input(id, src.as_slice(), |b, src| {
                b.iter(|| f(black_box(src), black_box(dst.as_mut_slice())))
            });
        }
    }
}

criterion_group!(benches, bench_encode, bench_decode, bench_check, bench_forgiving_decode);
criterion_main!(benches);
