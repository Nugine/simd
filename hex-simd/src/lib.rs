#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(aarch64_target_feature))]
#![cfg_attr(feature = "unstable", feature(arm_target_feature))]
//
#![deny(clippy::all, clippy::cargo)]
#![warn(clippy::todo)]
#![allow(clippy::missing_safety_doc)] // TODO

#[cfg(feature = "alloc")]
extern crate alloc;

pub(crate) use simd_abstraction::common::hex as sa_hex;

pub use sa_hex::AsciiCase;
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
