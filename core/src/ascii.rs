//! Operations on ASCII strings and characters.
//!
//! Most string operations in Rust act on UTF-8 strings. However, at times it
//! makes more sense to only consider the ASCII character set for a specific
//! operation.
//!
//! The [`escape_default`] function provides an iterator over the bytes of an
//! escaped version of the character given.

#![stable(feature = "core_ascii", since = "1.26.0")]

use crate::escape;
use crate::fmt;
use crate::iter::FusedIterator;
use crate::num::NonZeroUsize;

/// An iterator over the escaped version of a byte.
///
/// This `struct` is created by the [`escape_default`] function. See its
/// documentation for more.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Clone)]
pub struct EscapeDefault(escape::EscapeIterInner<4>);

/// Returns an iterator that produces an escaped version of a `u8`.
///
/// The default is chosen with a bias toward producing literals that are
/// legal in a variety of languages, including C++11 and similar C-family
/// languages. The exact rules are:
///
/// * Tab is escaped as `\t`.
/// * Carriage return is escaped as `\r`.
/// * Line feed is escaped as `\n`.
/// * Single quote is escaped as `\'`.
/// * Double quote is escaped as `\"`.
/// * Backslash is escaped as `\\`.
/// * Any character in the 'printable ASCII' range `0x20` .. `0x7e`
///   inclusive is not escaped.
/// * Any other chars are given hex escapes of the form '\xNN'.
/// * Unicode escapes are never generated by this function.
///
/// # Examples
///
/// ```
/// use std::ascii;
///
/// let escaped = ascii::escape_default(b'0').next().unwrap();
/// assert_eq!(b'0', escaped);
///
/// let mut escaped = ascii::escape_default(b'\t');
///
/// assert_eq!(b'\\', escaped.next().unwrap());
/// assert_eq!(b't', escaped.next().unwrap());
///
/// let mut escaped = ascii::escape_default(b'\r');
///
/// assert_eq!(b'\\', escaped.next().unwrap());
/// assert_eq!(b'r', escaped.next().unwrap());
///
/// let mut escaped = ascii::escape_default(b'\n');
///
/// assert_eq!(b'\\', escaped.next().unwrap());
/// assert_eq!(b'n', escaped.next().unwrap());
///
/// let mut escaped = ascii::escape_default(b'\'');
///
/// assert_eq!(b'\\', escaped.next().unwrap());
/// assert_eq!(b'\'', escaped.next().unwrap());
///
/// let mut escaped = ascii::escape_default(b'"');
///
/// assert_eq!(b'\\', escaped.next().unwrap());
/// assert_eq!(b'"', escaped.next().unwrap());
///
/// let mut escaped = ascii::escape_default(b'\\');
///
/// assert_eq!(b'\\', escaped.next().unwrap());
/// assert_eq!(b'\\', escaped.next().unwrap());
///
/// let mut escaped = ascii::escape_default(b'\x9d');
///
/// assert_eq!(b'\\', escaped.next().unwrap());
/// assert_eq!(b'x', escaped.next().unwrap());
/// assert_eq!(b'9', escaped.next().unwrap());
/// assert_eq!(b'd', escaped.next().unwrap());
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub fn escape_default(c: u8) -> EscapeDefault {
    let mut data = [0; 4];
    let range = escape::escape_ascii_into(&mut data, c);
    EscapeDefault(escape::EscapeIterInner::new(data, range))
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Iterator for EscapeDefault {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.len();
        (n, Some(n))
    }

    #[inline]
    fn count(self) -> usize {
        self.0.len()
    }

    #[inline]
    fn last(mut self) -> Option<u8> {
        self.0.next_back()
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        self.0.advance_by(n)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl DoubleEndedIterator for EscapeDefault {
    #[inline]
    fn next_back(&mut self) -> Option<u8> {
        self.0.next_back()
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        self.0.advance_back_by(n)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ExactSizeIterator for EscapeDefault {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for EscapeDefault {}

#[stable(feature = "ascii_escape_display", since = "1.39.0")]
impl fmt::Display for EscapeDefault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for EscapeDefault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EscapeDefault").finish_non_exhaustive()
    }
}
