use criterion::{black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use uuid::Uuid;
use uuid_simd::{AsciiCase, OutRef, UuidExt};

pub fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid-simd-parse");

    let inputs = [
        ("simple", "67e5504410b1426f9247bb680e5fe0c8"),
        ("hyphenated", "67e55044-10b1-426f-9247-bb680e5fe0c8"),
        ("guid", "{67e55044-10b1-426f-9247-bb680e5fe0c8}"),
        ("urn", "urn:uuid:67e55044-10b1-426f-9247-bb680e5fe0c8"),
    ];

    #[allow(clippy::type_complexity)]
    let functions: &[(&str, fn(&str))] = &[
        ("uuid-simd/auto-indirect", |s: &str| {
            <Uuid as UuidExt>::parse(s).unwrap();
        }),
        ("uuid/fallback", |s: &str| {
            Uuid::try_parse(s).unwrap();
        }),
    ];

    for &(name, f) in functions {
        for (tag, input) in inputs {
            group.bench_with_input(BenchmarkId::new(name, tag), input, |b, s| b.iter(|| f(black_box(s))));
        }
    }
}

pub fn bench_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid-simd-format");

    let x = Uuid::from_bytes(0x67e5504410b1426f9247bb680e5fe0c8_u128.to_be_bytes());

    macro_rules! wrap {
        ($b: expr, $u: expr, $f: expr) => {{
            let mut ans = 0;
            $b.iter(|| {
                #[allow(clippy::redundant_closure_call)]
                let dst = ($f)(black_box($u));
                let dst = black_box(&dst);
                ans = ans ^ (ans * dst.last().unwrap());
            });
            assert!(ans != 1);
        }};
    }

    {
        #[allow(clippy::type_complexity)]
        let functions: &[(&str, fn(&mut Bencher, &Uuid))] = &[
            ("uuid-simd/auto-indirect", |b: &mut Bencher, u: &Uuid| {
                wrap!(b, u, |u: &Uuid| {
                    let mut buf = [0; 32];
                    let src = u.as_bytes();
                    let dst = OutRef::from_mut(&mut buf);
                    let _ = uuid_simd::format_simple(src, dst, AsciiCase::Lower);
                    buf
                });
            }),
            ("uuid/fallback", |b: &mut Bencher, u: &Uuid| {
                wrap!(b, u, |u: &Uuid| {
                    let mut buf = [0; 32];
                    u.as_simple().encode_lower(&mut buf);
                    buf
                });
            }),
        ];

        for &(name, f) in functions {
            group.bench_with_input(BenchmarkId::new(name, "simple-lowercase"), &x, f);
        }
    }

    {
        #[allow(clippy::type_complexity)]
        let functions: &[(&str, fn(&mut Bencher, &Uuid))] = &[
            ("uuid-simd/auto-indirect", |b: &mut Bencher, u: &Uuid| {
                wrap!(b, u, |u: &Uuid| {
                    let mut buf = [0; 36];
                    let src = u.as_bytes();
                    let dst = OutRef::from_mut(&mut buf);
                    let _ = uuid_simd::format_hyphenated(src, dst, AsciiCase::Lower);
                    buf
                });
            }),
            ("uuid/fallback", |b: &mut Bencher, u: &Uuid| {
                wrap!(b, u, |u: &Uuid| {
                    let mut buf = [0; 36];
                    u.as_hyphenated().encode_lower(&mut buf);
                    buf
                });
            }),
        ];

        for &(name, f) in functions {
            group.bench_with_input(BenchmarkId::new(name, "hyphenated-lowercase"), &x, f);
        }
    }
}

criterion_group!(benches, bench_parse, bench_format);
criterion_main!(benches);
