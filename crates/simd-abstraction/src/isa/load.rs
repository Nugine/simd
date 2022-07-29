use super::{InstructionSet, SIMD128, SIMD256};

use crate::scalar::{Bytes16, Bytes32};

pub unsafe trait SimdLoad<T>: InstructionSet {
    type Output;

    fn load(self, src: T) -> Self::Output;
}

unsafe impl<S: SIMD128> SimdLoad<&'_ Bytes16> for S {
    type Output = S::V128;

    #[inline(always)]
    fn load(self, src: &'_ Bytes16) -> Self::Output {
        unsafe { self.v128_load(src.0.as_ptr()) }
    }
}

unsafe impl<S: SIMD256> SimdLoad<&'_ Bytes32> for S {
    type Output = S::V256;

    #[inline(always)]
    fn load(self, src: &'_ Bytes32) -> Self::Output {
        unsafe { self.v256_load(src.0.as_ptr()) }
    }
}
