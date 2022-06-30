use crate::traits::{SIMD128, SIMD256};

use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

/// A write-only slice of bytes.
pub struct OutBuf<'a> {
    data: NonNull<MaybeUninit<u8>>,
    len: usize,
    _marker: PhantomData<&'a mut [MaybeUninit<u8>]>,
}

unsafe impl<'a> Send for OutBuf<'a> {}
unsafe impl<'a> Sync for OutBuf<'a> {}

impl<'a> OutBuf<'a> {
    /// Returns an `OutBuf<'a>`
    ///
    /// # Safety
    /// This function requires:
    ///
    /// + It's safe to call `slice::from_raw_parts_mut(data, len)`
    ///
    /// See also [`slice::from_raw_parts_mut`](core::slice::from_raw_parts_mut)
    ///
    #[inline]
    pub unsafe fn from_raw(data: *mut MaybeUninit<u8>, len: usize) -> Self {
        Self {
            data: NonNull::new_unchecked(data),
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn new(slice: &'a mut [u8]) -> Self {
        let (data, len): _ = (slice.as_mut_ptr(), slice.len());
        unsafe { Self::from_raw(data.cast(), len) }
    }

    #[inline]
    pub fn uninit(slice: &'a mut [MaybeUninit<u8>]) -> Self {
        let (data, len): _ = (slice.as_mut_ptr(), slice.len());
        unsafe { Self::from_raw(data, len) }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.data.as_ptr().cast()
    }
}

impl<'a> From<&'a mut [u8]> for OutBuf<'a> {
    #[inline(always)]
    fn from(slice: &'a mut [u8]) -> Self {
        Self::new(slice)
    }
}

impl<'a> From<&'a mut [MaybeUninit<u8>]> for OutBuf<'a> {
    #[inline(always)]
    fn from(slice: &'a mut [MaybeUninit<u8>]) -> Self {
        Self::uninit(slice)
    }
}

impl fmt::Debug for OutBuf<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OutBuf")
            .field("data", &self.data)
            .field("len", &self.len)
            .finish()
    }
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

#[allow(unused_macros)]
macro_rules! debug_assert_ptr_align {
    ($ptr:expr, $align:literal) => {{
        let align: usize = $align;
        let ptr = $ptr as *const _ as *const ();
        let addr = ptr as usize;
        debug_assert!(addr % align == 0)
    }};
}

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

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
    use core::slice;
    let layout = Layout::from_size_align_unchecked(len, 1);
    let p = alloc(layout);
    if p.is_null() {
        handle_alloc_error(layout)
    }
    let ptr = p.cast();
    Box::from_raw(slice::from_raw_parts_mut(ptr, len))
}