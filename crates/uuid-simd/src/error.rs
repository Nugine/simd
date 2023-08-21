use core::fmt;

/// UUID Error
pub struct Error(());

impl Error {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Error(())
    }
}

impl fmt::Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Debug>::fmt("UUIDError", f)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Display>::fmt("UUIDError", f)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for Error {}

macro_rules! ensure {
    ($cond:expr) => {
        if !$cond {
            return Err($crate::error::Error::new());
        }
    };
}

#[allow(unused_macros)]
macro_rules! try_ {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(()) => return Err(Error::new()),
        }
    };
}
