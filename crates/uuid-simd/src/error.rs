use core::fmt;

/// UUID Error
#[derive(Debug)]
pub struct Error(());

pub(crate) const ERROR: Error = Error(());

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Display>::fmt("UUIDError", f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
