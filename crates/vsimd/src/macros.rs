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

macro_rules! is_subtype {
    ($self:ident, $super:ident) => {{
        <$self as $crate::InstructionSet>::is_subtype_of::<$super>()
    }};
    ($self:ident, $($super:ident)|+) => {{
        false $(|| <$self as $crate::InstructionSet>::is_subtype_of::<$super>())+
    }};
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
                unused_unsafe,
                clippy::missing_safety_doc,
                clippy::must_use_candidate,
            )]

            use super::*;

            use $crate::isa::InstructionSet;

            const _: $(for<$($lifetime),+>)? $($unsafe)? fn($($arg_type),*) -> $ret = $($fallback_fn)+;

            #[inline]
            pub $($unsafe)? fn fallback$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                $($fallback_fn)+($($arg_name),*)
            }

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            $crate::item_group!{
                use $crate::{AVX2, SSE41};

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
                use $crate::NEON;

                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(NEON$(,$arg_type)*) -> $ret = $($simd_fn)+::<NEON>;

                #[inline]
                #[target_feature(enable = "neon")]
                pub unsafe fn neon$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(NEON::new() $(,$arg_name)*)
                }
            }

            #[cfg(target_arch = "wasm32")]
            $crate::item_group!{
                use $crate::WASM128;

                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(WASM128$(,$arg_type)*) -> $ret = $($simd_fn)+::<WASM128>;

                #[inline]
                #[target_feature(enable = "simd128")]
                pub unsafe fn simd128$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(WASM128::new() $(,$arg_name)*)
                }
            }

            #[inline(always)]
            fn resolve() -> $(for<$($lifetime),+>)? unsafe fn($($arg_type),*) -> $ret {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                if $crate::AVX2::is_enabled() {
                    return avx2;
                }
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                if $crate::SSE41::is_enabled() {
                    return sse41;
                }
                #[cfg(all(feature = "unstable", any(target_arch = "arm",target_arch = "aarch64")))]
                if $crate::NEON::is_enabled() {
                    return neon;
                }
                #[cfg(target_arch = "wasm32")]
                if $crate::WASM128::is_enabled() {
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
