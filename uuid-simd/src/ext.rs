use crate::{AsciiCase, Error};

use core::fmt;

use uuid::Uuid;

/// An extension trait for [`uuid::Uuid`]
pub trait UuidExt: Sized {
    /// Parses an UUID from arbitrary bytes.
    ///
    /// # Errors
    /// This function returns `Err` if:
    ///
    /// + The length of `src` doesn't match any UUID format variants.
    /// + The content of `src` is invalid.
    ///
    fn parse(src: impl AsRef<[u8]>) -> Result<Self, Error>;

    /// Parses a simple UUID from arbitrary bytes.
    ///
    /// # Errors
    /// This function returns `Err` if:
    ///
    /// + The length of `src` doesn't match the "simple" format.
    /// + The content of `src` is invalid.
    ///
    fn parse_simple(src: impl AsRef<[u8]>) -> Result<Self, Error>;

    /// Parses a hyphenated UUID from arbitrary bytes.
    ///
    /// # Errors
    /// This function returns `Err` if:
    ///
    /// + The length of `src` doesn't match the "hyphenated" format.
    /// + The content of `src` is invalid.
    ///
    fn parse_hyphenated(src: impl AsRef<[u8]>) -> Result<Self, Error>;

    /// Returns a fmt adapter with "simple" format.
    fn format_simple(&self) -> Simple<'_>;

    /// Returns a fmt adapter with "hyphenated" format.
    fn format_hyphenated(&self) -> Hyphenated<'_>;
}

impl UuidExt for Uuid {
    #[inline]
    fn parse(src: impl AsRef<[u8]>) -> Result<Self, Error> {
        crate::parse(src.as_ref()).map(Uuid::from_bytes)
    }

    #[inline]
    fn parse_simple(src: impl AsRef<[u8]>) -> Result<Self, Error> {
        crate::parse_simple(src.as_ref()).map(Uuid::from_bytes)
    }

    #[inline]
    fn parse_hyphenated(src: impl AsRef<[u8]>) -> Result<Self, Error> {
        crate::parse_hyphenated(src.as_ref()).map(Uuid::from_bytes)
    }

    fn format_simple(&self) -> Simple<'_> {
        Simple(self)
    }

    fn format_hyphenated(&self) -> Hyphenated<'_> {
        Hyphenated(self)
    }
}

pub struct Simple<'a>(&'a Uuid);

pub struct Hyphenated<'a>(&'a Uuid);

impl fmt::LowerHex for Simple<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf = crate::format_simple(self.0.as_bytes(), AsciiCase::Lower);
        <&str as fmt::Display>::fmt(&buf.as_str(), f)
    }
}

impl fmt::LowerHex for Hyphenated<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf = crate::format_hyphenated(self.0.as_bytes(), AsciiCase::Lower);
        <&str as fmt::Display>::fmt(&buf.as_str(), f)
    }
}

impl fmt::UpperHex for Simple<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf = crate::format_simple(self.0.as_bytes(), AsciiCase::Upper);
        <&str as fmt::Display>::fmt(&buf.as_str(), f)
    }
}

impl fmt::UpperHex for Hyphenated<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf = crate::format_hyphenated(self.0.as_bytes(), AsciiCase::Upper);
        <&str as fmt::Display>::fmt(&buf.as_str(), f)
    }
}

impl fmt::Display for Simple<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::LowerHex>::fmt(self, f)
    }
}

impl fmt::Display for Hyphenated<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::LowerHex>::fmt(self, f)
    }
}

#[test]
fn test() {
    let s1 = "67e5504410b1426f9247bb680e5fe0c8";
    let s2 = "67e55044-10b1-426f-9247-bb680e5fe0c8";

    let u = Uuid::parse(s1).unwrap();

    let a1 = u.format_simple().to_string();
    let a2 = format!("{:X}", u.format_hyphenated());

    assert_eq!(a1, s1);
    assert_eq!(a2, s2.to_ascii_uppercase());
}
