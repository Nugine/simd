use crate::{V128, V256, V512};

pub unsafe trait POD: Copy {}

macro_rules! mark_pod {
    ($($ty:ty),*) => {
        $(
            unsafe impl POD for $ty {}
        )*
    };
}

mark_pod!(u8, u16, u32, u64, u128, usize);
mark_pod!(i8, i16, i32, i64, i128, isize);
mark_pod!(f32, f64);
mark_pod!(V128, V256, V512);

#[inline(always)]
pub fn align<T: POD, U: POD>(slice: &[T]) -> (&[T], &[U], &[T]) {
    unsafe { slice.align_to() }
}
