# simd

[![MIT licensed][mit-badge]][mit-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: ./LICENSE

SIMD-accelerated operations

|                crate                 |                                                version                                                |                                      docs                                      |
| :----------------------------------: | :---------------------------------------------------------------------------------------------------: | :----------------------------------------------------------------------------: |
| [base64-simd](./crates/base64-simd/) | [![Crates.io](https://img.shields.io/crates/v/base64-simd.svg)](https://crates.io/crates/base64-simd) | [![Docs](https://docs.rs/base64-simd/badge.svg)](https://docs.rs/base64-simd/) |
|    [hex-simd](./crates/hex-simd/)    |    [![Crates.io](https://img.shields.io/crates/v/hex-simd.svg)](https://crates.io/crates/hex-simd)    |    [![Docs](https://docs.rs/hex-simd/badge.svg)](https://docs.rs/hex-simd/)    |
|   [uuid-simd](./crates/uuid-simd/)   |   [![Crates.io](https://img.shields.io/crates/v/uuid-simd.svg)](https://crates.io/crates/uuid-simd)   |   [![Docs](https://docs.rs/uuid-simd/badge.svg)](https://docs.rs/uuid-simd/)   |

The crates provide fast fallback implementations and automatically select SIMD functions when available.

## Safety

This project relies heavily on unsafe code. We encourage everyone to review the code and report any issues.

Memory safety bugs and unsoundness issues are classified as critical bugs. They will be fixed as soon as possible.
