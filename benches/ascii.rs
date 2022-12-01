use simd_benches::{map_collect, rand_bytes, FnGroup};

use criterion::{black_box, criterion_group, criterion_main, AxisScale, PlotConfiguration};
use criterion::{BenchmarkId, Criterion, Throughput};

fn gen_ascii(len: usize) -> Vec<u8> {
    let mut buf = rand_bytes(len);
    let to_ascii = |x: &mut u8| *x /= 2;
    buf.iter_mut().for_each(to_ascii);
    buf
}

pub fn bench_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("ascii-check");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let cases = [3, 4, 8, 16, 31, 32, 64, 256, 1024, 4096, 1000000];
    let inputs: Vec<Vec<u8>> = map_collect(cases, gen_ascii);

    let functions: &FnGroup<fn(&[u8])> = &[
        ("unicode-simd/auto", |src: &[u8]| {
            assert!(unicode_simd::is_ascii(src));
        }),
        ("encoding_rs/auto", |src: &[u8]| {
            assert!(encoding_rs::mem::is_ascii(src)); //
        }),
        ("std/fallback", |src: &[u8]| {
            assert!(src.is_ascii());
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

criterion_group!(benches, bench_check);
criterion_main!(benches);
