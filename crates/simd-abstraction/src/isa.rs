#![allow(clippy::missing_safety_doc)]

mod load;

#[macro_use]
mod simd128;

#[macro_use]
mod simd256;

pub mod mock256;

pub use self::load::SimdLoad;
pub use self::simd128::SIMD128;
pub use self::simd256::SIMD256;

pub unsafe trait InstructionSet: Copy {
    fn is_enabled() -> bool;

    unsafe fn new() -> Self;

    #[inline(always)]
    fn detect() -> Option<Self> {
        Self::is_enabled().then(|| unsafe { Self::new() })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fallback(());

unsafe impl InstructionSet for Fallback {
    #[inline(always)]
    fn is_enabled() -> bool {
        true
    }

    #[inline(always)]
    unsafe fn new() -> Self {
        Self(())
    }
}

macro_rules! define_isa {
    ($ty:ident, $feature: tt) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $ty(());

        unsafe impl InstructionSet for $ty {
            #[inline(always)]
            fn is_enabled() -> bool {
                cfg!(target_feature = $feature)
            }

            #[inline(always)]
            unsafe fn new() -> Self {
                Self(())
            }
        }
    };

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
