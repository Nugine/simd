use core::mem::transmute;

#[cfg(feature = "unstable")]
use core::simd::{u8x16, u8x32, u8x64};

#[cfg(not(feature = "unstable"))]
#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct V128([u8; 16]);

#[cfg(not(feature = "unstable"))]
#[derive(Debug, Clone, Copy)]
#[repr(C, align(32))]
pub struct V256([u8; 32]);

#[cfg(not(feature = "unstable"))]
#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct V512([u8; 64]);

#[cfg(feature = "unstable")]
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct V128(u8x16);

#[cfg(feature = "unstable")]
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct V256(u8x32);

#[cfg(feature = "unstable")]
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct V512(u8x64);

impl V128 {
    #[inline(always)]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        unsafe { transmute(bytes) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        unsafe { transmute(self) }
    }
}

impl V256 {
    #[inline(always)]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        unsafe { transmute(bytes) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        unsafe { transmute(self) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn from_v128x2(x: (V128, V128)) -> Self {
        unsafe { transmute([x.0, x.1]) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn to_v128x2(self) -> (V128, V128) {
        let x: [V128; 2] = unsafe { transmute(self) };
        (x[0], x[1])
    }

    #[inline(always)]
    #[must_use]
    pub const fn double_bytes(bytes: [u8; 16]) -> Self {
        unsafe { transmute([bytes, bytes]) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn double_v128(x: V128) -> Self {
        unsafe { transmute([x, x]) }
    }
}

impl V512 {
    #[inline(always)]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 64]) -> Self {
        unsafe { transmute(bytes) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 64] {
        unsafe { transmute(self) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn from_v256x2(x: (V256, V256)) -> Self {
        unsafe { transmute([x.0, x.1]) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn to_v256x2(self) -> (V256, V256) {
        let x: [V256; 2] = unsafe { transmute(self) };
        (x[0], x[1])
    }

    #[inline(always)]
    #[must_use]
    pub const fn double_bytes(bytes: [u8; 32]) -> Self {
        unsafe { transmute([bytes, bytes]) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn double_v256(x: V256) -> Self {
        unsafe { transmute([x, x]) }
    }
}
