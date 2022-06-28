use core::fmt;

/// Hex Error
#[derive(Debug)]
pub struct Error(());

pub(crate) const ERROR: Error = Error(());

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Display>::fmt("HexError", f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
