use crate::traits::{SIMD128, SIMD256};

#[cfg(feature = "alloc")]
item_group! {
    use core::mem::MaybeUninit;
    use alloc::boxed::Box;
}

#[derive(Debug)]
#[repr(C, align(16))]
pub struct Bytes16(pub [u8; 16]);

#[derive(Debug)]
#[repr(C, align(32))]
pub struct Bytes32(pub [u8; 32]);

pub trait Load<T> {
    type Output;

    fn load(self, src: T) -> Self::Output;
}

impl<S: SIMD128> Load<&'_ Bytes16> for S {
    type Output = S::V128;

    #[inline(always)]
    fn load(self, src: &'_ Bytes16) -> Self::Output {
        unsafe { self.v128_load(src.0.as_ptr()) }
    }
}

impl<S: SIMD256> Load<&'_ Bytes32> for S {
    type Output = S::V256;

    #[inline(always)]
    fn load(self, src: &'_ Bytes32) -> Self::Output {
        unsafe { self.v256_load(src.0.as_ptr()) }
    }
}

/// Allocates uninit bytes
///
/// # Safety
/// This function requires:
///
/// + `len > 0`
/// + `len <= isize::MAX`
///
#[cfg(feature = "alloc")]
#[inline]
pub unsafe fn alloc_uninit_bytes(len: usize) -> Box<[MaybeUninit<u8>]> {
    #[cfg(any(debug_assertions, miri))]
    {
        assert!(len > 0 && len <= (isize::MAX as usize))
    }
    use alloc::alloc::{alloc, handle_alloc_error, Layout};
    let layout = Layout::from_size_align_unchecked(len, 1);
    let p = alloc(layout);
    if p.is_null() {
        handle_alloc_error(layout)
    }
    let ptr = p.cast();
    Box::from_raw(core::slice::from_raw_parts_mut(ptr, len))
}

#[allow(clippy::missing_safety_doc)]
#[cfg(feature = "alloc")]
#[inline]
pub unsafe fn assume_init(b: Box<[MaybeUninit<u8>]>) -> Box<[u8]> {
    let len = b.len();
    let ptr = Box::into_raw(b).cast::<u8>();
    Box::from_raw(core::ptr::slice_from_raw_parts_mut(ptr, len))
}

#[allow(clippy::missing_safety_doc)]
#[inline(always)]
pub unsafe fn read<T>(base: *const T, offset: usize) -> T {
    base.add(offset).read()
}

#[allow(clippy::missing_safety_doc)]
#[inline(always)]
pub unsafe fn write<T>(base: *mut T, offset: usize, value: T) {
    base.add(offset).write(value)
}

#[allow(clippy::missing_safety_doc)]
#[inline(always)]
pub unsafe fn slice_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T] {
    core::slice::from_raw_parts_mut(data, len)
}

#[inline(always)]
pub fn unroll<T>(slice: &[T], chunk_size: usize, mut f: impl FnMut(&T)) {
    let mut iter = slice.chunks_exact(chunk_size);
    for chunk in &mut iter {
        chunk.iter().for_each(&mut f)
    }
    iter.remainder().iter().for_each(&mut f);
}
