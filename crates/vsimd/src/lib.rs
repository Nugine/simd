//! ⚠️ This crate contains shared implementation details. Do not directly depend on it.
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(all(feature = "unstable", not(target_arch = "powerpc64")), feature(portable_simd))]
#![cfg_attr(
    all(feature = "unstable", target_arch = "arm"),
    feature(arm_target_feature),
    feature(stdarch_arm_feature_detection),
    feature(stdarch_arm_neon_intrinsics)
)]
#![cfg_attr(
    all(feature = "unstable", target_arch = "powerpc64"),
    feature(stdarch_powerpc),
    feature(stdarch_powerpc_feature_detection),
    feature(powerpc_target_feature)
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]
//
#![deny(
    missing_debug_implementations,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
    clippy::missing_inline_in_public_items
)]
#![warn(clippy::todo)]
#![allow(
    clippy::inline_always,
    missing_docs,
    clippy::missing_safety_doc,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::verbose_bit_mask,
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::items_after_statements,
    clippy::match_same_arms,
    clippy::many_single_char_names,
    clippy::let_underscore_untyped
)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod macros;

#[macro_use]
pub mod isa;

pub mod vector;

#[macro_use]
pub mod pod;
pub use self::pod::POD;

pub mod simulation;
mod unified;

mod simd64;
pub use self::simd64::SIMD64;

mod simd128;
pub use self::simd128::SIMD128;

#[macro_use]
mod simd256;
pub use self::simd256::SIMD256;

mod scalable;
pub use self::scalable::Scalable;

pub mod tools;

#[macro_use]
pub mod alsw;

pub mod ascii;
pub mod bswap;
pub mod hex;
pub mod mask;
pub mod native;
pub mod table;

#[cfg(all(feature = "unstable", not(target_arch = "powerpc64")))]
pub mod unstable;
