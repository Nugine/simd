use crate::tools::is_same_type;
use crate::{SIMD128, SIMD256, SIMD64};

pub unsafe trait InstructionSet: Copy + 'static {
    unsafe fn new() -> Self;

    fn is_enabled() -> bool;

    fn is_subtype_of<T: InstructionSet>() -> bool;

    #[inline(always)]
    #[must_use]
    fn detect() -> Option<Self> {
        Self::is_enabled().then(|| unsafe { Self::new() })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fallback(());

unsafe impl InstructionSet for Fallback {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        true
    }

    #[inline(always)]
    fn is_subtype_of<T: InstructionSet>() -> bool {
        is_same_type::<Self, T>()
    }
}

#[allow(unused_macros)]
macro_rules! is_feature_detected {
    ($feature:tt) => {{
        #[cfg(target_feature = $feature)]
        {
            true
        }
        #[cfg(not(target_feature = $feature))]
        {
            #[cfg(feature = "detect")]
            {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                {
                    std::arch::is_x86_feature_detected!($feature)
                }
                #[cfg(target_arch = "arm")]
                {
                    std::arch::is_arm_feature_detected!($feature)
                }
                #[cfg(target_arch = "aarch64")]
                {
                    std::arch::is_aarch64_feature_detected!($feature)
                }
                #[cfg(not(any(
                    target_arch = "x86",
                    target_arch = "x86_64",
                    target_arch = "arm",
                    target_arch = "aarch64"
                )))]
                {
                    false
                }
            }
            #[cfg(not(feature = "detect"))]
            {
                false
            }
        }
    }};
}

macro_rules! x86_is_enabled {
    ($feature:tt) => {{
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            is_feature_detected!($feature)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            false
        }
    }};
}

#[derive(Debug, Clone, Copy)]
pub struct SSE2(());

unsafe impl InstructionSet for SSE2 {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        x86_is_enabled!("sse2")
    }

    #[inline(always)]
    fn is_subtype_of<T: InstructionSet>() -> bool {
        is_same_type::<Self, T>() || Fallback::is_subtype_of::<T>()
    }
}

unsafe impl SIMD64 for SSE2 {}
unsafe impl SIMD128 for SSE2 {}
unsafe impl SIMD256 for SSE2 {}

#[derive(Debug, Clone, Copy)]
pub struct SSSE3(());

unsafe impl InstructionSet for SSSE3 {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        x86_is_enabled!("ssse3")
    }

    #[inline(always)]
    fn is_subtype_of<T: InstructionSet>() -> bool {
        is_same_type::<Self, T>() || SSE2::is_subtype_of::<T>()
    }
}

unsafe impl SIMD64 for SSSE3 {}
unsafe impl SIMD128 for SSSE3 {}
unsafe impl SIMD256 for SSSE3 {}

#[derive(Debug, Clone, Copy)]
pub struct SSE41(());

unsafe impl InstructionSet for SSE41 {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        x86_is_enabled!("sse4.1")
    }

    #[inline(always)]
    fn is_subtype_of<T: InstructionSet>() -> bool {
        is_same_type::<Self, T>() || SSSE3::is_subtype_of::<T>()
    }
}

unsafe impl SIMD64 for SSE41 {}
unsafe impl SIMD128 for SSE41 {}
unsafe impl SIMD256 for SSE41 {}

#[derive(Debug, Clone, Copy)]
pub struct AVX2(());

unsafe impl InstructionSet for AVX2 {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        x86_is_enabled!("avx2")
    }

    #[inline(always)]
    fn is_subtype_of<T: InstructionSet>() -> bool {
        is_same_type::<Self, T>() || SSE41::is_subtype_of::<T>()
    }
}

unsafe impl SIMD64 for AVX2 {}
unsafe impl SIMD128 for AVX2 {}
unsafe impl SIMD256 for AVX2 {}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub struct NEON(());

unsafe impl InstructionSet for NEON {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        #[cfg(target_arch = "arm")]
        {
            #[cfg(feature = "unstable")]
            {
                is_feature_detected!("neon")
            }
            #[cfg(not(feature = "unstable"))]
            {
                false
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            is_feature_detected!("neon")
        }
        #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
        {
            false
        }
    }

    #[inline(always)]
    fn is_subtype_of<T: InstructionSet>() -> bool {
        is_same_type::<Self, T>() || Fallback::is_subtype_of::<T>()
    }
}

unsafe impl SIMD64 for NEON {}
unsafe impl SIMD128 for NEON {}
unsafe impl SIMD256 for NEON {}

#[derive(Debug, Clone, Copy)]
pub struct WASM128(());

unsafe impl InstructionSet for WASM128 {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            is_feature_detected!("simd128")
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            false
        }
    }

    #[inline(always)]
    fn is_subtype_of<T: InstructionSet>() -> bool {
        is_same_type::<Self, T>() || Fallback::is_subtype_of::<T>()
    }
}

unsafe impl SIMD64 for WASM128 {}
unsafe impl SIMD128 for WASM128 {}
unsafe impl SIMD256 for WASM128 {}

#[cfg(test)]
mod tests {
    use super::*;

    use core::ops::Not;

    #[test]
    fn subtyping() {
        macro_rules! assert_subtype {
            ($ty:ident: $super: ident) => {
                assert!(is_subtype!($ty, $ty));
                assert!(is_subtype!($super, $super));
                assert!(is_subtype!($ty, $super));
                assert!(is_subtype!($super, $ty).not());
            };
        }

        assert_subtype!(SSE2: Fallback);

        assert_subtype!(SSSE3: Fallback);
        assert_subtype!(SSSE3: SSE2);

        assert_subtype!(SSE41: Fallback);
        assert_subtype!(SSE41: SSE2);
        assert_subtype!(SSE41: SSSE3);

        assert_subtype!(AVX2: Fallback);
        assert_subtype!(AVX2: SSE2);
        assert_subtype!(AVX2: SSSE3);
        assert_subtype!(AVX2: SSE41);

        assert_subtype!(NEON: Fallback);

        assert_subtype!(WASM128: Fallback);
    }
}
