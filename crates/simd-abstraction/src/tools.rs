#[cfg(feature = "alloc")]
item_group! {
    use core::mem::MaybeUninit;
    use alloc::boxed::Box;
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
#[must_use]
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

#[cfg(feature = "alloc")]
#[inline]
#[must_use]
pub unsafe fn assume_init(b: Box<[MaybeUninit<u8>]>) -> Box<[u8]> {
    let len = b.len();
    let ptr = Box::into_raw(b).cast::<u8>();
    Box::from_raw(core::ptr::slice_from_raw_parts_mut(ptr, len))
}

#[inline(always)]
pub unsafe fn read<T>(base: *const T, offset: usize) -> T {
    base.add(offset).read()
}

#[inline(always)]
pub unsafe fn write<T>(base: *mut T, offset: usize, value: T) {
    base.add(offset).write(value)
}

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
