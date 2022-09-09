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
pub use self::isa::*;

mod vector;
pub use self::vector::*;

mod simd128;
pub use self::simd128::*;

mod simd256;
pub use self::simd256::*;

pub mod scalar;
pub mod tools;

pub mod ascii;
pub mod bswap;
pub mod mask;
