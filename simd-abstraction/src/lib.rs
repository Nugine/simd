#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
//
#![deny(missing_debug_implementations, clippy::all, clippy::cargo)]
#![warn(clippy::todo)]
#![allow(clippy::missing_safety_doc)] // TODO

#[macro_export]
macro_rules! item_group {
    ($($item:item)*) => {
        $($item)*
    }
}

#[macro_use]
pub mod tools;

#[macro_use]
pub mod traits;

pub mod arch {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub mod x86;

    #[cfg(all(feature = "unstable", target_arch = "arm"))]
    pub mod arm;

    #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
    pub mod aarch64;

    #[cfg(target_arch = "wasm32")]
    pub mod wasm;
}

pub mod common {
    pub mod hex;
}