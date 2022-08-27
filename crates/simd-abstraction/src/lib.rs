//! Abstract SIMD instruction sets.
//!
//! ⚠️ This crate contains shared implementation details. Do not directly depend on it.
//!
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
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

#[macro_use]
pub mod isa;

pub mod scalar;

pub mod tools;

pub mod common {
    pub mod ascii;
    pub mod base32;
    pub mod bswap;
    pub mod crc32;
    pub mod hex;
}

pub mod arch {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub mod x86;

    #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
    pub mod arm;

    #[cfg(target_arch = "wasm32")]
    pub mod wasm;
}
