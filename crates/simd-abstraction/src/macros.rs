#[macro_export]
macro_rules! item_group {
    ($($item:item)*) => {
        $($item)*
    }
}

macro_rules! debug_assert_ptr_align {
    ($ptr:expr, $align:literal) => {{
        let align: usize = $align;
        let ptr = $ptr as *const _ as *const ();
        let addr = ptr as usize;
        debug_assert!(addr % align == 0)
    }};
}

#[allow(unused_macros)]
macro_rules! define_isa {
    ($ty:ident, $feature: tt, $detect: tt) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $ty(());

        unsafe impl InstructionSet for $ty {
            #[inline(always)]
            fn is_enabled() -> bool {
                #[cfg(target_feature = $feature)]
                {
                    true
                }
                #[cfg(not(target_feature = $feature))]
                {
                    #[cfg(feature = "detect")]
                    if std::arch::$detect!($feature) {
                        return true;
                    }
                    false
                }
            }

            #[inline(always)]
            unsafe fn new() -> Self {
                Self(())
            }
        }
    };
}

#[macro_export]
macro_rules! simd_dispatch {
    (
        name        = $name:ident,
        signature   = $(for<$($lifetime:lifetime),+>)? fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty,
        fallback    = {$($fallback_fn:tt)+},
        simd        = {$($simd_fn:tt)+},
        safety      = {$($unsafe:ident)?},
    ) => {
        pub mod $name {
            #![allow(
                unsafe_op_in_unsafe_fn,
                clippy::missing_safety_doc,
            )]

            use super::*;

            use $crate::traits::InstructionSet;

            const _: $(for<$($lifetime),+>)? $($unsafe)? fn($($arg_type),*) -> $ret = $($fallback_fn)+;

            #[inline]
            pub $($unsafe)? fn fallback$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                $($fallback_fn)+($($arg_name),*)
            }

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            $crate::item_group!{
                use $crate::arch::x86::{AVX2, SSE41};

                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(AVX2 $(,$arg_type)*) -> $ret = $($simd_fn)+::<AVX2>;
                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(SSE41$(,$arg_type)*) -> $ret = $($simd_fn)+::<SSE41>;

                #[inline]
                #[target_feature(enable = "avx2")]
                pub unsafe fn avx2$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(AVX2::new() $(,$arg_name)*)
                }

                #[inline]
                #[target_feature(enable = "sse4.1")]
                pub unsafe fn sse41$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(SSE41::new() $(,$arg_name)*)
                }
            }

            #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
            $crate::item_group!{
                #[cfg(target_arch = "aarch64")]
                use $crate::arch::aarch64::NEON;
                #[cfg(target_arch = "arm")]
                use $crate::arch::arm::NEON;

                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(NEON$(,$arg_type)*) -> $ret = $($simd_fn)+::<NEON>;

                #[inline]
                #[target_feature(enable = "neon")]
                pub unsafe fn neon$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(NEON::new() $(,$arg_name)*)
                }
            }

            #[cfg(target_arch = "wasm32")]
            $crate::item_group!{
                use $crate::arch::wasm;

                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(wasm::SIMD128$(,$arg_type)*) -> $ret = $($simd_fn)+::<wasm::SIMD128>;

                #[inline]
                #[target_feature(enable = "simd128")]
                pub unsafe fn simd128$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(wasm::SIMD128::new() $(,$arg_name)*)
                }
            }

            #[inline(always)]
            fn resolve() -> $(for<$($lifetime),+>)? unsafe fn($($arg_type),*) -> $ret {
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
                $($fallback_fn)+
            }

            use core::sync::atomic::{AtomicPtr, Ordering::Relaxed};

            static IFUNC: AtomicPtr<()> = AtomicPtr::new(init_ifunc as *mut ());

            $($unsafe)? fn init_ifunc$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                let f = resolve();
                IFUNC.store(f as *mut (), Relaxed);
                unsafe { f($($arg_name),*) }
            }

            #[inline(always)]
            pub $($unsafe)? fn auto_indirect$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                unsafe {
                    let f: unsafe fn($($arg_type),*) -> $ret = core::mem::transmute(IFUNC.load(Relaxed));
                    f($($arg_name),*)
                }
            }
        }
    }
}
