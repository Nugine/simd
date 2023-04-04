# simd

[![MIT licensed][mit-badge]][mit-url] [English](./README.md) | [中文](./README.zh-CN.md)

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: ./LICENSE

SIMD 加速操作

|                crate                 |                                                version                                                |                                      docs                                      |
| :----------------------------------: | :---------------------------------------------------------------------------------------------------: | :----------------------------------------------------------------------------: |
| [base64-simd](./crates/base64-simd/) | [![Crates.io](https://img.shields.io/crates/v/base64-simd.svg)](https://crates.io/crates/base64-simd) | [![Docs](https://docs.rs/base64-simd/badge.svg)](https://docs.rs/base64-simd/) |
|    [hex-simd](./crates/hex-simd/)    |    [![Crates.io](https://img.shields.io/crates/v/hex-simd.svg)](https://crates.io/crates/hex-simd)    |    [![Docs](https://docs.rs/hex-simd/badge.svg)](https://docs.rs/hex-simd/)    |
|   [uuid-simd](./crates/uuid-simd/)   |   [![Crates.io](https://img.shields.io/crates/v/uuid-simd.svg)](https://crates.io/crates/uuid-simd)   |   [![Docs](https://docs.rs/uuid-simd/badge.svg)](https://docs.rs/uuid-simd/)   |

这些 crate 自动选择可用的 SIMD 函数并提供快速的回退实现。基准测试结果可在 [simd-benches](https://github.com/Nugine/simd-benches) 查看。

## 目标

+ 性能：做到最快
+ 生产力：高效的 SIMD 抽象
+ 人体工程学：易于使用

## 安全性

本项目高度依赖不安全的代码。我们鼓励每个人审查代码并报告任何问题。

内存安全错误和健全性问题被归类为致命错误。它们将被尽快修复。

## 语言

本项目接受中文或英文。所有代码、文档、PR 和议题都应该使用中文或英文编写。

## 参考资料

本项目包含多种算法和实现。其中一些不是原创的。我们在这里列出参考资料。

base64:

+ <http://0x80.pl/articles/index.html#base64-algorithm-new>
+ <https://gist.github.com/aqrit/a2ccea48d7cac7e9d4d99f19d4759666>

hex:

+ <http://0x80.pl/notesen/2022-01-17-validating-hex-parse.html>

unicode:

+ <https://github.com/simdutf/simdutf>

## 赞助

如果我的开源工作对您有帮助，请[赞助我](https://github.com/Nugine#sponsor)。

每一点点都有帮助。非常感谢！
