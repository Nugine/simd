//! ⚠️ This crate contains shared implementation details. Do not directly depend on it.
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(
    feature = "unstable",
    feature(stdsimd),
    feature(arm_target_feature),
    feature(portable_simd)
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//
#![deny(
    missing_debug_implementations,
    clippy::all,
    clippy::cargo,
    clippy::missing_inline_in_public_items,
    clippy::must_use_candidate
)]
#![warn(clippy::todo)]
#![allow(
    clippy::missing_safety_doc, // TODO
)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod macros;

mod isa;
pub use self::isa::{InstructionSet, AVX2, NEON, SSE41, WASM128};

mod vector;
pub use self::vector::{V128, V256, V512, V64};

mod simd128;
pub use self::simd128::SIMD128;

mod simd256;
pub use self::simd256::SIMD256;

#[macro_use]
mod algorithm;

pub mod pod;
pub mod tools;

#[macro_use]
pub mod alsw;

pub mod ascii;
pub mod base32;
pub mod base64;
pub mod bswap;
pub mod hex;
pub mod mask;
pub mod table;
