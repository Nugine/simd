#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(feature = "unstable", feature(stdsimd))]
//
#![deny(
    missing_debug_implementations,
    clippy::all,
    clippy::cargo,
    clippy::missing_inline_in_public_items
)]
#![warn(clippy::todo)]

#[cfg(feature = "alloc")]
extern crate alloc;

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

pub mod ascii;
pub mod crc32;
pub mod hex;

#[macro_export]
macro_rules! simd_dispatch {
    {
        $name:ident = fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty,
        fallback = $fallback_fn: path,
        simd = $simd_fn: ident,
    } => {
        pub mod $name {
            #![allow(clippy::missing_safety_doc)]

            use super::*;

            use $crate::traits::InstructionSet;

            const _: fn($($arg_type),*) -> $ret = $fallback_fn;

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            $crate::item_group!{
                #[target_feature(enable = "avx2")]
                pub unsafe fn avx2($($arg_name:$arg_type),*) -> $ret {
                    use $crate::arch::x86::AVX2;
                    const _: fn(AVX2 $(,$arg_type)*) -> $ret = $simd_fn::<AVX2>;
                    $simd_fn(AVX2::new() $(,$arg_name)*)
                }

                #[target_feature(enable = "sse4.1")]
                pub unsafe fn sse41($($arg_name:$arg_type),*) -> $ret {
                    use $crate::arch::x86::SSE41;
                    const _: fn(SSE41$(,$arg_type)*) -> $ret = $simd_fn::<SSE41>;
                    $simd_fn(SSE41::new() $(,$arg_name)*)
                }
            }

            #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
            $crate::item_group!{
                #[target_feature(enable = "neon")]
                pub unsafe fn neon($($arg_name:$arg_type),*) -> $ret {
                    #[cfg(target_arch = "aarch64")]
                    use $crate::arch::aarch64::NEON;
                    #[cfg(target_arch = "arm")]
                    use $crate::arch::arm::NEON;

                    const _: fn(NEON$(,$arg_type)*) -> $ret = $simd_fn::<NEON>;
                    $simd_fn(NEON::new() $(,$arg_name)*)
                }
            }

            #[cfg(target_arch = "wasm32")]
            $crate::item_group!{
                #[target_feature(enable = "simd128")]
                pub unsafe fn simd128($($arg_name:$arg_type),*) -> $ret {
                    use $crate::arch::wasm;
                    const _: fn(wasm::SIMD128$(,$arg_type)*) -> $ret = $simd_fn::<wasm::SIMD128>;
                    $simd_fn(wasm::SIMD128::new() $(,$arg_name)*)
                }
            }

            #[inline(always)]
            fn resolve() -> unsafe fn($($arg_type),*) -> $ret {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                if $crate::arch::x86::AVX2::is_enabled() {
                    return avx2;
                }
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                if $crate::arch::x86::SSE41::is_enabled() {
                    return sse41;
                }
                #[cfg(all(feature = "unstable", target_arch = "aarch64"))]
                if $crate::arch::aarch64::NEON::is_enabled() {
                    return neon;
                }
                #[cfg(all(feature = "unstable", target_arch = "arm"))]
                if $crate::arch::arm::NEON::is_enabled() {
                    return neon;
                }
                #[cfg(target_arch = "wasm32")]
                if $crate::arch::wasm::SIMD128::is_enabled() {
                    return simd128;
                }
                $fallback_fn
            }

            #[inline(always)]
            pub fn auto_indirect($($arg_name:$arg_type),*) -> $ret {
                use core::sync::atomic::{AtomicPtr, Ordering::Relaxed};

                static IFUNC: AtomicPtr<()> = AtomicPtr::new(init as *mut ());

                fn init($($arg_name:$arg_type),*) -> $ret {
                    let f = resolve();
                    IFUNC.store(f as *mut (), Relaxed);
                    unsafe { f($($arg_name),*) }
                }

                unsafe {
                    let f: unsafe fn($($arg_type),*) -> $ret = core::mem::transmute(IFUNC.load(Relaxed));
                    f($($arg_name),*)
                }
            }
        }
    }
}
