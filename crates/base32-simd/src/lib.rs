//! SIMD-accelerated base32 encoding and decoding.

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
#![cfg_attr(docsrs, feature(doc_cfg))]
//
#![deny(
    missing_debug_implementations,
    missing_docs,
    clippy::all,
    clippy::cargo,
    clippy::missing_inline_in_public_items
)]
#![warn(clippy::todo)]
#![allow(dead_code, unused_macros)] // TODO

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod error;
