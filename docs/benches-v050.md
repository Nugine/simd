# Benchmarks v0.5.0

Rust Version: `rustc 1.59.0-nightly (b60e32c82 2021-12-30)`

CPU Name: `Intel(R) Core(TM) i5-9300H CPU @ 2.40GHz`

Benched with [Criterion.rs](https://github.com/bheisler/criterion.rs)

## uuid-simd

| Function                | Average Time |
| ----------------------- | ------------ |
| parse-simple(avx2)      | 5.3570ns     |
| parse-hyphenated(avx2)  | 7.3306 ns    |
| format-simple(avx2)     | 4.0277ns     |
| format-hyphenated(avx2) | 9.0357 ns    |

Average Time (Lower is better)

![violin plot](v050/uuid-simd-parse.svg)
![violin plot](v050/uuid-simd-format.svg)

## hex-simd

| Function     | Top Throughput |
| ------------ | -------------- |
| check(avx2)  | 21.542 GiB/s   |
| encode(avx2) | 16.370 GiB/s   |
| decode(avx2) | 10.347 GiB/s   |

Average Time (Lower is better)

![line chart](v050/hex-simd-check.svg)
![line chart](v050/hex-simd-encode.svg)
![line chart](v050/hex-simd-decode.svg)

## base64-simd

| Function     | Top Throughput |
| ------------ | -------------- |
| encode(avx2) | 9.4362 GiB/s   |
| decode(avx2) | 10.727 GiB/s   |

Average Time (Lower is better)

![line chart](v050/base64-simd-encode.svg)
![line chart](v050/base64-simd-decode.svg)
