#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct Bytes16(pub [u8; 16]);

#[derive(Debug, Clone)]
#[repr(C, align(32))]
pub struct Bytes32(pub [u8; 32]);

#[derive(Debug, Clone)]
#[repr(C, align(64))]
pub struct Bytes64(pub [u8; 64]);

impl Bytes32 {
    #[inline]
    #[must_use]
    pub const fn double(bytes16: [u8; 16]) -> Self {
        let mut bytes32 = [0u8; 32];
        let mut i = 0;
        while i < 16 {
            bytes32[i] = bytes16[i];
            bytes32[i + 16] = bytes16[i];
            i += 1;
        }
        Self(bytes32)
    }
}

pub unsafe trait Scalar: Copy {}

macro_rules! mark_scalar {
    ($($ty:ty),*) => {
        $(
            unsafe impl Scalar for $ty {}
        )*
    };
}

mark_scalar!(u8, u16, u32, u64, u128, usize);
mark_scalar!(i8, i16, i32, i64, i128, isize);
mark_scalar!(f32, f64);

#[inline(always)]
pub fn align16<T: Scalar>(slice: &[T]) -> (&[T], &[Bytes16], &[T]) {
    unsafe { slice.align_to() }
}

#[inline(always)]
pub fn align32<T: Scalar>(slice: &[T]) -> (&[T], &[Bytes32], &[T]) {
    unsafe { slice.align_to() }
}

#[inline(always)]
pub fn align64<T: Scalar>(slice: &[T]) -> (&[T], &[Bytes64], &[T]) {
    unsafe { slice.align_to() }
}

#[inline(always)]
pub fn align<T: Scalar, U: Scalar>(slice: &[T]) -> (&[T], &[U], &[T]) {
    unsafe { slice.align_to() }
}
