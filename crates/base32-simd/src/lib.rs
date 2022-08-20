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

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod error;

mod common;

mod rfc4648;
pub use self::rfc4648::{Rfc4648Base32, BASE32, BASE32HEX};

mod crockford;
pub use self::crockford::{CrockfordBase32, CROCKFORD_BASE32};

pub use simd_abstraction::OutBuf;
