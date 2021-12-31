#[inline(always)]
pub unsafe fn read<T>(base: *const T, offset: usize) -> T {
    base.add(offset).read()
}

#[inline(always)]
pub unsafe fn write<T>(base: *mut T, offset: usize, value: T) {
    base.add(offset).write(value)
}

#[inline(always)]
pub unsafe fn empty_slice_mut<'a, T>(base: *mut T) -> &'a mut [T] {
    core::slice::from_raw_parts_mut(base, 0)
}
