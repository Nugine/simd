//! SIMD-accelerated UUID operations.
//!
//! # Examples
//!
//! ```
//! use uuid::Uuid;
//! use uuid_simd::UuidExt;
//!
//! let text = "67e55044-10b1-426f-9247-bb680e5fe0c8";
//! let uuid: Uuid = Uuid::parse(text.as_bytes()).unwrap();
//! println!("{}", uuid.format_simple())
//! ```
//!

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
#![cfg_attr(feature = "unstable", feature(aarch64_target_feature))]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
//
#![deny(clippy::all, clippy::cargo, missing_docs)]
#![warn(clippy::todo)]

pub(crate) use simd_abstraction::common::hex as sa_hex;

pub use sa_hex::{AsciiCase, Hex};

mod error;
pub use self::error::Error;
pub(crate) use self::error::ERROR;

#[cfg(test)]
mod tests;

pub mod fallback;

#[macro_use]
mod generic;

mod polyfill;

pub mod arch;

mod auto;
pub use self::auto::*;

use simd_abstraction::item_group;

#[cfg(feature = "uuid")]
item_group! {
    mod ext;
    pub use self::ext::UuidExt;
}
