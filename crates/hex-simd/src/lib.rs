//! SIMD-accelerated hex encoding and decoding.
//!
//! # Examples
//!
//! ```
//! use hex_simd::AsciiCase;
//!
//! let bytes = b"Hello world!";
//!
//! let encoded = hex_simd::encode_to_boxed_str(bytes, AsciiCase::Lower);
//! assert_eq!(&*encoded, "48656c6c6f20776f726c6421");
//!
//! let decoded = hex_simd::decode_to_boxed_bytes(encoded.as_bytes()).unwrap();
//! assert_eq!(&*decoded, bytes);
//! ```
//!

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
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

pub(crate) use simd_abstraction::common::hex as sa_hex;

pub use simd_abstraction::ascii::AsciiCase;
pub use simd_abstraction::tools::OutBuf;

mod error;
pub use self::error::Error;
pub(crate) use self::error::ERROR;

#[cfg(test)]
mod tests;

pub mod fallback;

#[macro_use]
mod generic;

pub mod arch;

mod auto;
pub use self::auto::*;

mod ext;
pub use self::ext::*;
