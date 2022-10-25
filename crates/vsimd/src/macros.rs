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

#[macro_export]
macro_rules! dispatch_v2 {
    (
        name        = {$name:ident},
        signature   = {$vis:vis unsafe fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty},
        fallback    = {$fallback_fn:path},
        simd        = {$simd_fn:path},
        targets     = {$($target:tt),+},
        fastest     = {$($fastest:tt),*},
    ) => {
        $vis mod $name {
            #![allow(
                clippy::missing_safety_doc,
                clippy::must_use_candidate,
            )]

            use super::*;

            use $crate::SIMD256;

            #[allow(dead_code)]
            #[inline]
            $vis unsafe fn simd<S: SIMD256>(s: S $(,$arg_name: $arg_type)*) -> $ret {
                $simd_fn(s, $($arg_name),*)
            }

            $crate::dispatch_v2!(
                @iter_compile,
                signature   = {$vis unsafe fn($($arg_name: $arg_type),*) -> $ret},
                simd        = {$simd_fn},
                targets     = {$($target),+},
            );

            #[allow(unreachable_code)]
            #[cfg(not(all(feature = "detect", not(target_arch = "wasm32"))))] // auto_direct
            #[inline]
            $vis unsafe fn auto($($arg_name: $arg_type),*) -> $ret {
                $crate::dispatch_v2!(
                    @iter_resolve_static,
                    targets     = {$($target),+},
                    args        = {$($arg_name),*},
                );
                $fallback_fn($($arg_name),*)
            }

            #[cfg(all(feature = "detect", not(target_arch = "wasm32")))] // auto_indirect
            $crate::item_group! {
                use core::sync::atomic::{AtomicPtr, Ordering::Relaxed};

                static IFUNC: AtomicPtr<()> = AtomicPtr::new(init_ifunc as *mut ());

                #[inline(always)]
                fn resolve() -> unsafe fn($($arg_type),*) -> $ret {
                    use $crate::isa::InstructionSet;
                    $crate::dispatch_v2!(@iter_resolve_dynamic, targets = {$($target),+},);
                    $fallback_fn
                }

                #[inline]
                unsafe fn init_ifunc($($arg_name: $arg_type),*) -> $ret {
                    let f = resolve();
                    IFUNC.store(f as *mut (), Relaxed);
                    f($($arg_name),*)
                }

                #[allow(unreachable_code)]
                #[inline(never)]
                $vis unsafe fn auto($($arg_name: $arg_type),*) -> $ret {
                    $crate::dispatch_v2!(
                        @iter_resolve_static,
                        targets     = {$($fastest),+},
                        args        = {$($arg_name),*},
                    );

                    let f: unsafe fn($($arg_type),*) -> $ret = core::mem::transmute(IFUNC.load(Relaxed));
                    f($($arg_name),*)
                }
            }
        }
    };

    (
        @iter_resolve_static,
        targets     = {$x:tt, $($xs:tt),+},
        args        = {$($arg_name: ident),*},
    ) => {
        $crate::dispatch_v2!(@resolve_static, $x, $($arg_name),*);
        $crate::dispatch_v2!(@iter_resolve_static, targets = {$($xs),+}, args = {$($arg_name),*},);
    };

    (
        @iter_resolve_static,
        targets     = {$x:tt},
        args        = {$($arg_name: ident),*},
    ) => {
        $crate::dispatch_v2!(@resolve_static, $x, $($arg_name),*);
    };

    (@resolve_static, "avx2", $($arg_name: ident),*) => {
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            return unsafe { avx2($($arg_name),*) }
        }
    };

    (@resolve_static, "sse4.1", $($arg_name: ident),*) => {
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse4.1"
        ))]
        {
            return unsafe { sse41($($arg_name),*) }
        }
    };

    (@resolve_static, "ssse3", $($arg_name: ident),*) => {
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "ssse3"
        ))]
        {
            return unsafe { ssse3($($arg_name),*) }
        }
    };

    (@resolve_static, "sse2", $($arg_name: ident),*) => {
        #[cfg(all(
            not(miri),
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2"
        ))]
        {
            return unsafe { sse2($($arg_name),*) }
        }
    };

    (@resolve_static, "neon", $($arg_name: ident),*) => {
        #[cfg(all(
            feature = "unstable",
            any(target_arch = "arm", target_arch = "aarch64"),
            target_feature = "neon",
        ))]
        {
            return unsafe { neon($($arg_name),*) }
        }
    };

    (@resolve_static, "simd128", $($arg_name: ident),*) => {
        #[cfg(all(
            target_arch = "wasm32",
            target_feature = "simd128",
        ))]
        {
            return unsafe { simd128($($arg_name),*) }
        }
    };

    (
        @iter_resolve_dynamic,
        targets     = {$x:tt, $($xs:tt),+},
    ) => {
        $crate::dispatch_v2!(@resolve_dynamic, $x);
        $crate::dispatch_v2!(@iter_resolve_dynamic, targets = {$($xs),+},);
    };

    (
        @iter_resolve_dynamic,
        targets     = {$x:tt},
    ) => {
        $crate::dispatch_v2!(@resolve_dynamic, $x);
    };

    (@resolve_dynamic, "avx2") => {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if $crate::isa::AVX2::is_enabled() {
            return avx2;
        }
    };

    (@resolve_dynamic, "sse4.1") => {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if $crate::isa::SSE41::is_enabled() {
            return sse41;
        }
    };

    (@resolve_dynamic, "ssse3") => {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if $crate::isa::SSSE3::is_enabled() {
            return ssse3;
        }
    };

    (@resolve_dynamic, "sse2") => {
        #[cfg(all(not(miri), any(target_arch = "x86", target_arch = "x86_64")))]
        if $crate::isa::SSE2::is_enabled() {
            return sse2;
        }
    };

    (@resolve_dynamic, "neon") => {
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        if $crate::isa::NEON::is_enabled() {
            return neon;
        }
    };

    (@resolve_dynamic, "simd128") => {
        #[cfg(target_arch = "wasm32")]
        if $crate::isa::WASM128::is_enabled() {
            return simd128;
        }
    };

    (
        @iter_compile,
        signature   = {$vis:vis unsafe fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty},
        simd        = {$simd_fn:path},
        targets     = {$x:tt, $($xs:tt),+},
    ) => {
        $crate::dispatch_v2!(
            @compile,
            signature   = {$vis unsafe fn($($arg_name: $arg_type),*) -> $ret},
            simd        = {$simd_fn},
            target      = {$x},
        );

        $crate::dispatch_v2!(
            @iter_compile,
            signature   = {$vis unsafe fn($($arg_name: $arg_type),*) -> $ret},
            simd        = {$simd_fn},
            targets     = {$($xs),+},
        );
    };

    (
        @iter_compile,
        signature   = {$vis:vis unsafe fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty},
        simd        = {$simd_fn:path},
        targets     = {$x:tt},
    ) => {
        $crate::dispatch_v2!(
            @compile,
            signature   = {$vis unsafe fn($($arg_name: $arg_type),*) -> $ret},
            simd        = {$simd_fn},
            target      = {$x},
        );
    };

    (
        @compile,
        signature   = {$vis:vis unsafe fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty},
        simd        = {$simd_fn:path},
        target      = {"avx2"},
    ) => {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        #[inline]
        #[target_feature(enable = "avx2")]
        $vis unsafe fn avx2($($arg_name:$arg_type),*) -> $ret {
            use $crate::isa::{AVX2, InstructionSet as _};
            $simd_fn(AVX2::new() $(,$arg_name)*)
        }
    };

    (
        @compile,
        signature   = {$vis:vis unsafe fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty},
        simd        = {$simd_fn:path},
        target      = {"sse4.1"},
    ) => {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        #[inline]
        #[target_feature(enable = "sse4.1")]
        $vis unsafe fn sse41($($arg_name:$arg_type),*) -> $ret {
            use $crate::isa::{SSE41, InstructionSet as _};
            $simd_fn(SSE41::new() $(,$arg_name)*)
        }
    };

    (
        @compile,
        signature   = {$vis:vis unsafe fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty},
        simd        = {$simd_fn:path},
        target      = {"ssse3"},
    ) => {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        #[inline]
        #[target_feature(enable = "ssse3")]
        $vis unsafe fn ssse3($($arg_name:$arg_type),*) -> $ret {
            use $crate::isa::{SSSE3, InstructionSet as _};
            $simd_fn(SSSE3::new() $(,$arg_name)*)
        }
    };

    (
        @compile,
        signature   = {$vis:vis unsafe fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty},
        simd        = {$simd_fn:path},
        target      = {"sse2"},
    ) => {
        #[cfg(all(not(miri), any(target_arch = "x86", target_arch = "x86_64")))]
        #[inline]
        #[target_feature(enable = "sse2")]
        $vis unsafe fn sse2($($arg_name:$arg_type),*) -> $ret {
            use $crate::isa::{SSE2, InstructionSet as _};
            $simd_fn(SSE2::new() $(,$arg_name)*)
        }
    };

    (
        @compile,
        signature   = {$vis:vis unsafe fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty},
        simd        = {$simd_fn:path},
        target      = {"neon"},
    ) => {
        #[cfg(all(feature = "unstable", any(target_arch = "arm", target_arch = "aarch64")))]
        #[inline]
        #[target_feature(enable = "neon")]
        $vis unsafe fn neon($($arg_name:$arg_type),*) -> $ret {
            use $crate::isa::{NEON, InstructionSet as _};
            $simd_fn(NEON::new() $(,$arg_name)*)
        }
    };

    (
        @compile,
        signature   = {$vis:vis unsafe fn($($arg_name: ident: $arg_type: ty),*) -> $ret:ty},
        simd        = {$simd_fn:path},
        target      = {"simd128"},
    ) => {
        #[cfg(target_arch = "wasm32")]
        #[inline]
        #[target_feature(enable = "simd128")]
        $vis unsafe fn simd128($($arg_name:$arg_type),*) -> $ret {
            use $crate::isa::{WASM128, InstructionSet as _};
            $simd_fn(WASM128::new() $(,$arg_name)*)
        }
    }
}
