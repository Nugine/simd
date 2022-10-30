use base32_simd::AsOut;
use simd_benches::FnGroup;
use uuid_simd::{AsciiCase, UuidExt};

use std::mem::MaybeUninit;

use const_str::hex;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use uuid::Uuid;

pub fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid-parse");

    let inputs = [
        ("simple", "67e5504410b1426f9247bb680e5fe0c8"),
        ("hyphenated", "67e55044-10b1-426f-9247-bb680e5fe0c8"),
        ("guid", "{67e55044-10b1-426f-9247-bb680e5fe0c8}"),
        ("urn", "urn:uuid:67e55044-10b1-426f-9247-bb680e5fe0c8"),
    ];

    let functions: &FnGroup<fn(&str) -> Uuid> = &[
        ("uuid-simd/auto", |s: &str| <Uuid as UuidExt>::parse(s).unwrap()),
        ("uuid/fallback", |s: &str| Uuid::try_parse(s).unwrap()),
    ];

    for &(name, f) in functions {
        for (tag, input) in inputs {
            group.bench_with_input(BenchmarkId::new(name, tag), input, |b, s| {
                b.iter(|| black_box(f(black_box(s))))
            });
        }
    }
}

pub fn bench_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid-format");

    let input = Uuid::from_bytes(hex!("67e5504410b1426f9247bb680e5fe0c8"));

    {
        let functions: &FnGroup<fn(&Uuid) -> [u8; 32]> = &[
            ("uuid-simd/auto", |u: &Uuid| {
                let mut buf: MaybeUninit<[u8; 32]> = MaybeUninit::uninit();
                *uuid_simd::format_simple(u.as_bytes(), buf.as_out(), AsciiCase::Lower)
            }),
            ("uuid/fallback", |u: &Uuid| {
                let mut buf = [0; 32];
                u.as_simple().encode_lower(&mut buf);
                buf
            }),
        ];

        for &(name, f) in functions {
            let tag = "simple";
            group.bench_with_input(BenchmarkId::new(name, tag), &input, |b, u| {
                b.iter(|| black_box(f(black_box(u))))
            });
        }
    }

    {
        let functions: &FnGroup<fn(&Uuid) -> [u8; 36]> = &[
            ("uuid-simd/auto", |u: &Uuid| {
                let mut buf: MaybeUninit<[u8; 36]> = MaybeUninit::uninit();
                *uuid_simd::format_hyphenated(u.as_bytes(), buf.as_out(), AsciiCase::Lower)
            }),
            ("uuid/fallback", |u: &Uuid| {
                let mut buf = [0; 36];
                u.as_hyphenated().encode_lower(&mut buf);
                buf
            }),
        ];

        for &(name, f) in functions {
            let tag = "hyphenated";
            group.bench_with_input(BenchmarkId::new(name, tag), &input, |b, u| {
                b.iter(|| black_box(f(black_box(u))))
            });
        }
    }
}

criterion_group!(benches, bench_parse, bench_format);
criterion_main!(benches);
