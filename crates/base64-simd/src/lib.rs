//! SIMD-accelerated base64 encoding and decoding.
//!
//! # Examples
//!
//! ```
//! let bytes = b"hello world";
//! let base64 = base64_simd::STANDARD;
//!
//! let encoded = base64.encode_to_boxed_str(bytes);
//! assert_eq!(&*encoded, "aGVsbG8gd29ybGQ=");
//!
//! let decoded = base64.decode_to_boxed_bytes(encoded.as_bytes()).unwrap();
//! assert_eq!(&*decoded, bytes);
//! ```
//!

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
#![cfg_attr(docsrs, feature(doc_cfg))]
//
#![deny(
    missing_debug_implementations,
    missing_docs,
    clippy::all,
    clippy::cargo,
    clippy::missing_inline_in_public_items,
    clippy::must_use_candidate
)]
#![warn(clippy::todo)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod error;
pub use self::error::Error;

mod forgiving;

mod rfc4648;
pub use self::rfc4648::{Rfc4648Base64, STANDARD, URL_SAFE};

#[cfg(test)]
mod tests;

pub use outref::OutRef;
