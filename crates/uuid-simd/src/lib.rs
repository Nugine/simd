//! TODO

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
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

mod error;
pub use self::error::Error;

mod spec;

mod parse;
