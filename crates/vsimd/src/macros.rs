#[macro_export]
macro_rules! item_group {
    ($($item:item)*) => {
        $($item)*
    }
}

macro_rules! debug_assert_ptr_align {
    ($ptr:expr, $align:literal) => {{
        let align: usize = $align;
        let ptr = <*const _>::cast::<()>($ptr);
        let addr = ptr as usize;
        debug_assert!(addr % align == 0)
    }};
}

#[macro_export]
macro_rules! is_subtype {
    ($self:ident, $super:ident) => {{
        <$self as $crate::isa::InstructionSet>::is_subtype_of::<$super>()
    }};
    ($self:ident, $super:ident | $($other:ident)|+) => {{
        <$self as $crate::isa::InstructionSet>::is_subtype_of::<$super>()
         $(|| <$self as $crate::isa::InstructionSet>::is_subtype_of::<$other>())+
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
        visibility  = {$vis:vis},
    ) => {
        pub mod $name {
            #![allow(
                unsafe_op_in_unsafe_fn,
                unused_unsafe,
                clippy::missing_safety_doc,
                clippy::must_use_candidate,
            )]

            use super::*;

            #[cfg(any(
                any(target_arch = "x86", target_arch = "x86_64"),
                all(feature = "unstable", any(target_arch = "arm",target_arch = "aarch64")),
                target_arch = "wasm32"
            ))]
            use $crate::isa::InstructionSet;

            use $crate::SIMD256;

            const _: $(for<$($lifetime),+>)? $($unsafe)? fn($($arg_type),*) -> $ret = $($fallback_fn)+;

            #[inline]
            $vis $($unsafe)? fn fallback$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                $($fallback_fn)+($($arg_name),*)
            }

            #[allow(dead_code)]
            #[inline]
            $vis $($unsafe)? fn simd<$($($lifetime,)+)? S: SIMD256>(s: S, $($arg_name:$arg_type),*) -> $ret {
                $($simd_fn)+(s, $($arg_name),*)
            }

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            $crate::item_group!{
                use $crate::isa::{AVX2, SSE41};

                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(AVX2 $(,$arg_type)*) -> $ret = $($simd_fn)+::<AVX2>;
                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(SSE41$(,$arg_type)*) -> $ret = $($simd_fn)+::<SSE41>;

                #[inline]
                #[target_feature(enable = "avx2")]
                $vis unsafe fn avx2$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(AVX2::new() $(,$arg_name)*)
                }

                #[inline]
                #[target_feature(enable = "sse4.1")]
                $vis unsafe fn sse41$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(SSE41::new() $(,$arg_name)*)
                }
            }

            #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
            $crate::item_group!{
                use $crate::isa::NEON;

                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(NEON$(,$arg_type)*) -> $ret = $($simd_fn)+::<NEON>;

                #[inline]
                #[target_feature(enable = "neon")]
                $vis unsafe fn neon$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(NEON::new() $(,$arg_name)*)
                }
            }

            #[cfg(target_arch = "wasm32")]
            $crate::item_group!{
                use $crate::isa::WASM128;

                const _: $(for<$($lifetime),+>)? $($unsafe)? fn(WASM128$(,$arg_type)*) -> $ret = $($simd_fn)+::<WASM128>;

                #[inline]
                #[target_feature(enable = "simd128")]
                $vis unsafe fn simd128$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    $($simd_fn)+(WASM128::new() $(,$arg_name)*)
                }
            }

            #[cfg(any(
                any(target_arch = "x86", target_arch = "x86_64"),
                all(feature = "unstable", any(target_arch = "arm",target_arch = "aarch64")),
                target_arch = "wasm32"
            ))]
            $crate::item_group!{
                #[inline(always)]
                fn resolve() -> $(for<$($lifetime),+>)? unsafe fn($($arg_type),*) -> $ret {
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                    if $crate::isa::AVX2::is_enabled() {
                        return avx2;
                    }
                    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                    if $crate::isa::SSE41::is_enabled() {
                        return sse41;
                    }
                    #[cfg(all(feature = "unstable", any(target_arch = "arm",target_arch = "aarch64")))]
                    if $crate::isa::NEON::is_enabled() {
                        return neon;
                    }
                    #[cfg(target_arch = "wasm32")]
                    if $crate::isa::WASM128::is_enabled() {
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
                $vis $($unsafe)? fn auto_indirect$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    unsafe {
                        let f: unsafe fn($($arg_type),*) -> $ret = core::mem::transmute(IFUNC.load(Relaxed));
                        f($($arg_name),*)
                    }
                }

                #[inline(always)]
                $vis $($unsafe)? fn auto_direct$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                    let f = resolve();
                    unsafe { f($($arg_name),*) }
                }
            }

            #[inline(always)]
            $vis $($unsafe)? fn auto$(<$($lifetime),+>)?($($arg_name:$arg_type),*) -> $ret {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                if cfg!(target_feature = "avx2") {
                    return unsafe { avx2($($arg_name),*) }
                }

                #[cfg(all(feature = "unstable", any(target_arch = "arm",target_arch = "aarch64")))]
                if cfg!(target_feature = "neon") {
                    return unsafe { neon($($arg_name),*) }
                }

                #[cfg(target_arch = "wasm32")]
                if cfg!(target_feature = "simd128") {
                    return unsafe { simd128($($arg_name),*) }
                }

                #[cfg(any(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    all(feature = "unstable", any(target_arch = "arm",target_arch = "aarch64")),
                    target_arch = "wasm32"
                ))]
                {
                    #[cfg(feature = "detect")]
                    {
                        auto_indirect($($arg_name),*)
                    }
                    #[cfg(not(feature = "detect"))]
                    {
                        auto_direct($($arg_name),*)
                    }
                }
                #[cfg(not(any(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    all(feature = "unstable", any(target_arch = "arm",target_arch = "aarch64")),
                    target_arch = "wasm32"
                )))]
                {
                    $($fallback_fn)+($($arg_name),*)
                }
            }
        }
    }
}

#[macro_export]
macro_rules! shared_docs {
    () => {
        r#"
# CPU feature detection

The feature flag *detect* is enabled by default.

When the feature flag *detect* is enabled, the APIs will **test at runtime** whether **the CPU (and OS)** supports the required instruction set. The runtime detection will be skipped if the fastest implementation is already available at compile-time.

When the feature flag *detect* is disabled, the APIs will **test at compile-time** whether **the compiler flags** supports the required instruction set.

If the environment supports SIMD acceleration, the APIs will call SIMD functions under the hood. Otherwise, the APIs will call fallback functions.

Supported instruction sets:

+ SSE4.1
+ AVX2
+ ARM NEON (*unstable*)
+ AArch64 NEON (*unstable*)
+ WASM SIMD128

When the feature flag *unstable* is enabled, this crate requires the nightly toolchain to compile.

# `no_std` support

You can disable the default features to use this crate in a `no_std` environment.

You can enable the feature flag *alloc* if the environment supports heap allocation.

Currently the feature flag *detect* depends on the standard library. Dynamic CPU feature detection is not available in `no_std` environments.

# Profile settings

To ensure maximum performance, the following [profile settings](https://doc.rust-lang.org/cargo/reference/profiles.html#profile-settings) are recommended when compiling this crate:

```toml
opt-level = 3
lto = "fat"
codegen-units = 1
```
"#
    };
}
