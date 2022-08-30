//! ⚠️ This crate contains shared implementation details. Do not directly depend on it.
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
#![cfg_attr(feature = "unstable", feature(portable_simd))]
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
