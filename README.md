# simd

[![MIT licensed][mit-badge]][mit-url] [![CI][CI-badge][CI-url]
[English](./README.md) | [中文](./README.zh-CN.md)

[CI-badge]: https://github.com/Nugine/simd/actions/workflows/ci.yml/badge.svg
[CI-url]: https://github.com/Nugine/simd/actions/workflows/ci.yml
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: ./LICENSE

SIMD-accelerated operations

|                crate                 |                                                version                                                |                                      docs                                      |
| :----------------------------------: | :---------------------------------------------------------------------------------------------------: | :----------------------------------------------------------------------------: |
| [base64-simd](./crates/base64-simd/) | [![Crates.io](https://img.shields.io/crates/v/base64-simd.svg)](https://crates.io/crates/base64-simd) | [![Docs](https://docs.rs/base64-simd/badge.svg)](https://docs.rs/base64-simd/) |
|    [hex-simd](./crates/hex-simd/)    |    [![Crates.io](https://img.shields.io/crates/v/hex-simd.svg)](https://crates.io/crates/hex-simd)    |    [![Docs](https://docs.rs/hex-simd/badge.svg)](https://docs.rs/hex-simd/)    |
|   [uuid-simd](./crates/uuid-simd/)   |   [![Crates.io](https://img.shields.io/crates/v/uuid-simd.svg)](https://crates.io/crates/uuid-simd)   |   [![Docs](https://docs.rs/uuid-simd/badge.svg)](https://docs.rs/uuid-simd/)   |

The crates automatically select SIMD functions when available and provide fast fallback implementations. Benchmark results are available in [simd-benches](https://github.com/Nugine/simd-benches).

## Goals

+ Performance: To be the fastest
+ Productivity: Efficient SIMD abstractions
+ Ergonomics: Easy to use

## Safety

This project relies heavily on unsafe code. We encourage everyone to review the code and report any issues.

Memory safety bugs and unsoundness issues are classified as critical bugs. They will be fixed as soon as possible.

## References

This project contains multiple algorithms and implementations. Some of them are not original. We list the references here.

base64:

+ <http://0x80.pl/articles/index.html#base64-algorithm-new>
+ <https://gist.github.com/aqrit/a2ccea48d7cac7e9d4d99f19d4759666>

hex:

+ <http://0x80.pl/notesen/2022-01-17-validating-hex-parse.html>

unicode:

+ <https://github.com/simdutf/simdutf>

## Sponsor

If my open-source work has been helpful to you, please [sponsor me](https://github.com/Nugine#sponsor).

Every little bit helps. Thank you!
