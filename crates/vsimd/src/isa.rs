use crate::tools::is_same_type;
use crate::{SIMD128, SIMD256, SIMD64};

pub unsafe trait InstructionSet: Copy + 'static {
    unsafe fn new() -> Self;

    fn is_enabled() -> bool;

    #[inline(always)]
    #[must_use]
    fn detect() -> Option<Self> {
        Self::is_enabled().then(|| unsafe { Self::new() })
    }

    #[inline(always)]
    #[must_use]
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
                if std::arch::is_x86_feature_detected!($feature) {
                    return true;
                }
                #[cfg(target_arch = "arm")]
                if std::arch::is_arm_feature_detected!($feature) {
                    return true;
                }
                #[cfg(target_arch = "aarch64")]
                if std::arch::is_aarch64_feature_detected!($feature) {
                    return true;
                }
            }

            false
        }
    }};
}

#[derive(Debug, Clone, Copy)]
pub struct SSE41(());

#[derive(Debug, Clone, Copy)]
pub struct AVX2(());

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub struct NEON(());

#[derive(Debug, Clone, Copy)]
pub struct WASM128(());

unsafe impl InstructionSet for SSE41 {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            is_feature_detected!("sse4.1")
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            false
        }
    }
}

unsafe impl SIMD64 for SSE41 {}
unsafe impl SIMD128 for SSE41 {}
unsafe impl SIMD256 for SSE41 {}

unsafe impl InstructionSet for AVX2 {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            is_feature_detected!("avx2")
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            false
        }
    }

    #[inline(always)]
    fn is_subtype_of<T: InstructionSet>() -> bool {
        is_same_type::<Self, T>() || SSE41::is_subtype_of::<T>()
    }
}

unsafe impl SIMD64 for AVX2 {}
unsafe impl SIMD128 for AVX2 {}
unsafe impl SIMD256 for AVX2 {}

unsafe impl InstructionSet for NEON {
    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }

    #[inline(always)]
    fn is_enabled() -> bool {
        #[cfg(feature = "unstable")]
        {
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            {
                is_feature_detected!("neon")
            }
            #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
            {
                false
            }
        }
        #[cfg(not(feature = "unstable"))]
        {
            false
        }
    }
}

unsafe impl SIMD64 for NEON {}
unsafe impl SIMD128 for NEON {}
unsafe impl SIMD256 for NEON {}

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
}

unsafe impl SIMD64 for WASM128 {}
unsafe impl SIMD128 for WASM128 {}
unsafe impl SIMD256 for WASM128 {}
