#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
#![cfg_attr(feature = "unstable", feature(aarch64_target_feature))]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
//
#![deny(clippy::all, clippy::cargo)]
#![warn(clippy::todo)]
#![allow(clippy::missing_safety_doc)] // TODO

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
